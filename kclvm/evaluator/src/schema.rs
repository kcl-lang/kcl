use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use generational_arena::Index;
use kclvm_ast::ast;
use kclvm_ast::walker::TypedResultWalker;
use kclvm_primitives::{DefaultHashBuilder, IndexMap};
use kclvm_runtime::{schema_runtime_type, ConfigEntryOperationKind, ValueRef};
use scopeguard::defer;

use crate::lazy::{merge_variables_and_setters, LazyEvalScope, LazyEvalScopeRef};
use crate::proxy::{call_schema_body, call_schema_check};
use crate::rule::RuleEvalContext;
use crate::ty::type_pack_and_check;
use crate::{error as kcl_error, Proxy};
use crate::{Evaluator, INNER_LEVEL};

pub type SchemaBodyHandler =
    Arc<dyn Fn(&Evaluator, &SchemaEvalContextRef, &ValueRef, &ValueRef) -> ValueRef>;

pub type SchemaCheckHandler =
    Arc<dyn Fn(&Evaluator, &SchemaEvalContextRef, &ValueRef, &ValueRef, &ValueRef) -> ()>;

pub type SchemaEvalContextRef = Rc<RefCell<SchemaEvalContext>>;

/// Proxy functions represent the saved functions of the runtime its,
/// rather than executing KCL defined functions or plugin functions.
#[derive(Clone, Debug)]
pub struct SchemaEvalContext {
    pub node: Rc<ast::SchemaStmt>,
    pub scope: Option<LazyEvalScopeRef>,
    pub index: Index,
    pub parent: Option<Index>,
    pub mixins: Vec<Index>,
    pub value: ValueRef,
    pub config: ValueRef,
    pub config_meta: ValueRef,
    pub optional_mapping: ValueRef,
    pub is_sub_schema: bool,
}

impl SchemaEvalContext {
    #[inline]
    pub fn new_with_node(
        node: ast::SchemaStmt,
        index: Index,
        parent: Option<Index>,
        mixins: Vec<Index>,
    ) -> Self {
        Self {
            node: Rc::new(node),
            scope: None,
            index,
            parent,
            mixins,
            value: ValueRef::dict(None),
            config: ValueRef::dict(None),
            config_meta: ValueRef::dict(None),
            optional_mapping: ValueRef::dict(None),
            is_sub_schema: true,
        }
    }

    /// Reset schema evaluation context state.
    #[inline]
    pub fn snapshot(&self, config: ValueRef, config_meta: ValueRef) -> SchemaEvalContextRef {
        Rc::new(RefCell::new(Self {
            node: self.node.clone(),
            index: self.index,
            parent: self.parent,
            mixins: self.mixins.clone(),
            scope: None,
            value: ValueRef::dict(None),
            config,
            config_meta,
            optional_mapping: ValueRef::dict(None),
            is_sub_schema: true,
        }))
    }

    /// New a schema evaluation context with schema value and config.
    #[inline]
    pub fn new_with_value(&self, value: &ValueRef, config: &ValueRef) -> SchemaEvalContextRef {
        Rc::new(RefCell::new(Self {
            node: self.node.clone(),
            index: self.index,
            parent: self.parent,
            mixins: self.mixins.clone(),
            scope: None,
            value: value.clone(),
            config: config.clone(),
            config_meta: ValueRef::dict(None),
            optional_mapping: ValueRef::dict(None),
            is_sub_schema: true,
        }))
    }

    /// Pass value references from other schema eval context.
    /// Note that do not change the schema node.
    pub fn set_info_with_schema(&mut self, other: &SchemaEvalContext) {
        self.config = other.config.clone();
        self.config_meta = other.config_meta.clone();
        self.value = other.value.clone();
        self.optional_mapping = other.optional_mapping.clone();
        self.is_sub_schema = false;
        // Set lazy eval scope.
        if let Some(other) = &other.scope {
            if let Some(scope) = &self.scope {
                let mut scope = scope.borrow_mut();
                let other = other.borrow();
                scope.cache = other.cache.clone();
                scope.levels = other.levels.clone();
                scope.cal_times = other.cal_times.clone();
                scope.setters = other.setters.clone();
            } else {
                let other = other.borrow();
                self.scope = Some(Rc::new(RefCell::new(LazyEvalScope {
                    cache: other.cache.clone(),
                    levels: other.levels.clone(),
                    cal_times: other.cal_times.clone(),
                    setters: other.setters.clone(),
                })))
            }
        }
    }

    /// Pass value references from other rule eval context.
    /// Note that do not change the schema node.
    pub fn set_info_with_rule(&mut self, other: &RuleEvalContext) {
        self.config = other.config.clone();
        self.config_meta = other.config_meta.clone();
        self.value = other.value.clone();
        self.optional_mapping = other.optional_mapping.clone();
        // Note that for the host schema, it will evaluate the final value.
        self.is_sub_schema = true;
    }

    /// Update parent schema and mixin schema information in the current scope.
    pub fn get_parent_schema(
        s: &Evaluator,
        parent: &Option<Box<ast::Node<ast::Identifier>>>,
    ) -> Option<Index> {
        if let Some(parent) = parent {
            let func = s
                .walk_identifier_with_ctx(&parent.node, &ast::ExprContext::Load, None)
                .expect(kcl_error::RUNTIME_ERROR_MSG);
            if let Some(index) = func.try_get_proxy() {
                Some(index)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Update parent schema and mixin schema information
    pub fn get_mixin_schemas(
        s: &Evaluator,
        mixins: &[Box<ast::Node<ast::Identifier>>],
    ) -> Vec<Index> {
        let mut results = vec![];
        for mixin in mixins {
            let func = s
                .walk_identifier_with_ctx(&mixin.node, &ast::ExprContext::Load, None)
                .expect(kcl_error::RUNTIME_ERROR_MSG);
            if let Some(index) = func.try_get_proxy() {
                let frame = {
                    let frames = s.frames.borrow();
                    frames
                        .get(index)
                        .expect(kcl_error::INTERNAL_ERROR_MSG)
                        .clone()
                };
                if let Proxy::Schema(_) = &frame.proxy {
                    results.push(index);
                }
            }
        }
        results
    }

    /// Whether the attribute is the schema context.
    pub fn has_attr(s: &Evaluator, ctx: &SchemaEvalContextRef, name: &str) -> bool {
        for stmt in &ctx.borrow().node.body {
            if let ast::Stmt::SchemaAttr(attr) = &stmt.node {
                if attr.name.node == name {
                    return true;
                }
            }
        }
        if let Some(index) = ctx.borrow().parent {
            let frame = {
                let frames = s.frames.borrow();
                frames
                    .get(index)
                    .expect(kcl_error::INTERNAL_ERROR_MSG)
                    .clone()
            };
            if let Proxy::Schema(schema) = &frame.proxy {
                return SchemaEvalContext::has_attr(s, &schema.ctx, name);
            }
        }
        false
    }

    /// Get all attribute from schema
    pub fn get_attrs(s: &Evaluator, ctx: &SchemaEvalContextRef) -> Vec<(String, bool)> {
        let mut attrs = vec![];
        for stmt in &ctx.borrow().node.body {
            if let ast::Stmt::SchemaAttr(attr) = &stmt.node {
                attrs.push((attr.name.node.clone(), attr.is_optional));
            }
        }
        if let Some(index) = ctx.borrow().parent {
            let frame = {
                let frames = s.frames.borrow();
                frames
                    .get(index)
                    .expect(kcl_error::INTERNAL_ERROR_MSG)
                    .clone()
            };
            if let Proxy::Schema(schema) = &frame.proxy {
                attrs.extend(SchemaEvalContext::get_attrs(s, &schema.ctx));
            }
        }
        attrs
    }

    /// Whether the index signature is the schema context.
    pub fn has_index_signature(s: &Evaluator, ctx: &SchemaEvalContextRef) -> bool {
        if ctx.borrow().node.index_signature.is_some() {
            return true;
        }

        if let Some(index) = ctx.borrow().parent {
            let frame = {
                let frames = s.frames.borrow();
                frames
                    .get(index)
                    .expect(kcl_error::INTERNAL_ERROR_MSG)
                    .clone()
            };
            if let Proxy::Schema(schema) = &frame.proxy {
                return SchemaEvalContext::has_index_signature(s, &schema.ctx);
            }
        }
        false
    }

    #[inline]
    pub fn is_fit_config(s: &Evaluator, ctx: &SchemaEvalContextRef, value: &ValueRef) -> bool {
        if value.is_config() {
            let config = value.as_dict_ref();
            for (key, _) in &config.values {
                let no_such_attr =
                    !SchemaEvalContext::has_attr(s, ctx, key) && !key.starts_with('_');
                let has_index_signature = SchemaEvalContext::has_index_signature(s, ctx);
                if !has_index_signature && no_such_attr {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    /// Init the lazy scope used to cache the lazy evaluation result.
    pub fn init_lazy_scope(&mut self, s: &Evaluator, index: Option<Index>) {
        // TODO: cache the lazy scope cross different schema instances.
        let mut setters = IndexMap::with_hasher(DefaultHashBuilder::default());
        // Parent schema setters
        if let Some(idx) = self.parent {
            let frame = {
                let frames = s.frames.borrow();
                frames
                    .get(idx)
                    .expect(kcl_error::INTERNAL_ERROR_MSG)
                    .clone()
            };
            if let Proxy::Schema(schema) = &frame.proxy {
                let mut parent = schema.ctx.borrow_mut();
                parent.init_lazy_scope(s, Some(idx));
                if let Some(scope) = &parent.scope {
                    merge_variables_and_setters(
                        &mut self.value,
                        &mut setters,
                        &scope.borrow().setters,
                    );
                }
            }
        }
        // Self setters
        merge_variables_and_setters(
            &mut self.value,
            &mut setters,
            &s.emit_setters(&self.node.body, index),
        );
        // Mixin schema setters
        for idx in &self.mixins {
            let frame = {
                let frames = s.frames.borrow();
                frames
                    .get(*idx)
                    .expect(kcl_error::INTERNAL_ERROR_MSG)
                    .clone()
            };
            if let Proxy::Schema(schema) = &frame.proxy {
                let mut mixin = schema.ctx.borrow_mut();
                mixin.init_lazy_scope(s, Some(*idx));
                if let Some(scope) = &mixin.scope {
                    merge_variables_and_setters(
                        &mut self.value,
                        &mut setters,
                        &scope.borrow().setters,
                    );
                }
            }
        }
        self.scope = Some(Rc::new(RefCell::new(LazyEvalScope {
            setters,
            ..Default::default()
        })))
    }

    /// Get the value from the context.
    pub fn get_value(&self, s: &Evaluator, key: &str, pkgpath: &str, target: &str) -> ValueRef {
        if let Some(scope) = &self.scope {
            let value = {
                match self.value.get_by_key(key) {
                    Some(value) => value.clone(),
                    None => s.get_variable_in_pkgpath(key, pkgpath),
                }
            };
            // Deal in-place modify and return it self immediately.
            if key == target && {
                let scope = scope.borrow();
                !scope.is_backtracking(key) || scope.setter_len(key) <= 1
            } {
                value
            } else {
                let cached_value = {
                    let scope = scope.borrow();
                    scope.cache.get(key).cloned()
                };
                match cached_value {
                    Some(value) => value.clone(),
                    None => {
                        let setters = {
                            let scope = scope.borrow();
                            scope.setters.get(key).cloned()
                        };
                        match &setters {
                            Some(setters) if !setters.is_empty() => {
                                // Call all setters function to calculate the value recursively.
                                let level = {
                                    let scope = scope.borrow();
                                    *scope.levels.get(key).unwrap_or(&0)
                                };
                                let next_level = level + 1;
                                {
                                    let mut scope = scope.borrow_mut();
                                    scope.levels.insert(key.to_string(), next_level);
                                }
                                let n = setters.len();
                                let index = n - next_level;
                                if index >= n {
                                    value
                                } else {
                                    // Call setter function
                                    s.walk_schema_stmts_with_setter(
                                        &self.node.body,
                                        &setters[index],
                                    )
                                    .expect(kcl_error::INTERNAL_ERROR_MSG);
                                    {
                                        let mut scope = scope.borrow_mut();
                                        scope.levels.insert(key.to_string(), level);
                                        let value = match self.value.get_by_key(key) {
                                            Some(value) => value.clone(),
                                            None => s.undefined_value(),
                                        };
                                        scope.cache.insert(key.to_string(), value.clone());
                                        value
                                    }
                                }
                            }
                            _ => value,
                        }
                    }
                }
            }
        } else if let Some(value) = self.value.dict_get_value(key) {
            value
        } else {
            s.get_variable_in_pkgpath(key, pkgpath)
        }
    }

    /// Set value to the context.
    #[inline]
    pub fn set_value(&self, s: &Evaluator, key: &str) {
        if let Some(scope) = &self.scope {
            let mut scope = scope.borrow_mut();
            if (scope.cal_increment(key) || scope.is_last_setter_ast_index(key, &s.ast_id.borrow()))
                && scope.cache.get(key).is_none()
            {
                scope
                    .cache
                    .insert(key.to_string(), s.dict_get_value(&self.value, key));
            }
        }
    }
}

#[derive(Clone)]
pub struct SchemaCaller {
    pub ctx: SchemaEvalContextRef,
    pub body: SchemaBodyHandler,
    pub check: SchemaCheckHandler,
}

/// Init or reset the schema lazy eval scope.
pub(crate) fn init_lazy_scope(s: &Evaluator, ctx: &mut SchemaEvalContext) {
    let is_sub_schema = { ctx.is_sub_schema };
    let index = { ctx.index };
    if is_sub_schema {
        ctx.init_lazy_scope(s, Some(index));
    }
}

/// Schema body function
pub(crate) fn schema_body(
    s: &Evaluator,
    ctx: &SchemaEvalContextRef,
    args: &ValueRef,
    kwargs: &ValueRef,
) -> ValueRef {
    init_lazy_scope(s, &mut ctx.borrow_mut());
    // Schema self value or parent schema value;
    let mut schema_ctx_value = if let Some(parent_name) = &ctx.borrow().node.parent_name {
        let base_constructor_func = s.load_global_value(
            &parent_name.node.pkgpath,
            &parent_name
                .node
                .names
                .iter()
                .map(|n| n.node.as_str())
                .collect::<Vec<&str>>(),
        );
        // Call base schema function
        call_schema_body(s, &base_constructor_func, args, kwargs, ctx)
    } else {
        ctx.borrow().value.clone()
    };
    let schema_name = { ctx.borrow().node.name.node.to_string() };
    s.push_schema(crate::EvalContext::Schema(ctx.clone()));
    s.enter_schema_scope(true);
    defer! {
        s.leave_scope();
        s.pop_schema();
    }
    // Evaluate arguments and keyword arguments and store values to local variables.
    s.walk_arguments(&ctx.borrow().node.args, args, kwargs);
    // Eval schema body and record schema instances.
    {
        let schema_pkgpath = &s.current_pkgpath();
        // To prevent schema recursive calling, thus clone the AST here.
        let node = {
            let ctx = ctx.borrow();
            ctx.node.clone()
        };
        // Run schema compiled function
        for stmt in &node.body {
            s.walk_stmt(stmt).expect(kcl_error::RUNTIME_ERROR_MSG);
        }
        // Schema decorators check
        for decorator in &node.decorators {
            s.walk_decorator_with_name(&decorator.node, Some(&schema_name), true)
                .expect(kcl_error::RUNTIME_ERROR_MSG);
        }
        let runtime_type = kclvm_runtime::schema_runtime_type(&schema_name, schema_pkgpath);
        schema_ctx_value.set_potential_schema_type(&runtime_type);
        // Set schema arguments and keyword arguments
        schema_ctx_value.set_schema_args(args, kwargs);
    }
    // Schema Mixins
    {
        let ctx_ref = ctx.borrow();
        for mixin in &ctx_ref.node.mixins {
            let mixin_func = s.load_global_value(
                &mixin.node.pkgpath,
                &mixin
                    .node
                    .names
                    .iter()
                    .map(|n| n.node.as_str())
                    .collect::<Vec<&str>>(),
            );
            schema_ctx_value = call_schema_body(s, &mixin_func, args, kwargs, ctx);
        }
    }
    // Schema Attribute optional check
    let mut optional_mapping = { ctx.borrow().optional_mapping.clone() };
    {
        let ctx = ctx.borrow();
        for stmt in &ctx.node.body {
            if let ast::Stmt::SchemaAttr(schema_attr) = &stmt.node {
                s.dict_insert_value(
                    &mut optional_mapping,
                    &schema_attr.name.node,
                    &s.bool_value(schema_attr.is_optional),
                )
            }
        }
    }
    // Do schema check for the sub schema.
    let is_sub_schema = { ctx.borrow().is_sub_schema };
    let schema = if is_sub_schema {
        let index_sign_key_name = if let Some(index_signature) = &ctx.borrow().node.index_signature
        {
            if let Some(key_name) = &index_signature.node.key_name {
                key_name.node.clone()
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };
        if index_sign_key_name.is_empty() {
            // Update schema relaxed attribute
            update_schema_relaxed_attr(s, ctx, &mut schema_ctx_value);
            // Construct schema instance
            let schema = schema_with_config(s, ctx, &schema_ctx_value, args, kwargs);
            // Do schema optional attribute check recursively before evaluate check expressions.
            check_schema_optional_attr(s, &schema);
            // Call schema check block function
            schema_check(s, ctx, &schema, args, kwargs);
            schema
        } else {
            // Do check function for every index signature key
            let config = {
                let ctx = ctx.borrow();
                ctx.config.clone()
            };
            for (k, _) in &config.as_dict_ref().values {
                // relaxed keys
                if schema_ctx_value.attr_map_get(k).is_none() {
                    // Update index signature key value
                    let value = ValueRef::str(k);
                    schema_ctx_value.dict_update_key_value(&index_sign_key_name, value.clone());
                    // Update schema relaxed attribute
                    update_schema_relaxed_attr(s, ctx, &mut schema_ctx_value);
                    // Call schema check block function
                    schema_check(s, ctx, &schema_ctx_value, args, kwargs);
                }
            }
            schema_ctx_value.dict_remove(&index_sign_key_name);
            // Construct schema instance
            let schema = schema_with_config(s, ctx, &schema_ctx_value, args, kwargs);
            // Do schema optional attribute check recursively before evaluate check expressions.
            check_schema_optional_attr(s, &schema);
            schema
        }
    } else {
        // Record base schema instances.
        schema_with_config(s, ctx, &schema_ctx_value, args, kwargs)
    };
    schema
}

// Schema check and index sign value update function
pub(crate) fn schema_check(
    s: &Evaluator,
    ctx: &SchemaEvalContextRef,
    schema_value: &ValueRef,
    args: &ValueRef,
    kwargs: &ValueRef,
) {
    // Call base check function
    {
        let ctx_ref = ctx.borrow();
        if let Some(parent_name) = &ctx_ref.node.parent_name {
            let base_constructor_func = s
                .walk_identifier_with_ctx(&parent_name.node, &ast::ExprContext::Load, None)
                .expect(kcl_error::RUNTIME_ERROR_MSG);
            call_schema_check(
                s,
                &base_constructor_func,
                schema_value,
                args,
                kwargs,
                Some(ctx),
            )
        }
    }
    // Call self check function
    {
        let ctx = ctx.borrow();
        for check_expr in &ctx.node.checks {
            s.walk_check_expr(&check_expr.node)
                .expect(kcl_error::RUNTIME_ERROR_MSG);
        }
    }

    // Call mixin check functions
    {
        let ctx = ctx.borrow();
        for mixin in &ctx.node.mixins {
            let mixin_func = s
                .walk_identifier_with_ctx(&mixin.node, &ast::ExprContext::Load, None)
                .expect(kcl_error::RUNTIME_ERROR_MSG);
            if let Some(index) = mixin_func.try_get_proxy() {
                let frame = {
                    let frames = s.frames.borrow();
                    frames
                        .get(index)
                        .expect(kcl_error::INTERNAL_ERROR_MSG)
                        .clone()
                };
                if let Proxy::Schema(schema) = &frame.proxy {
                    s.push_pkgpath(&frame.pkgpath);
                    s.push_backtrace(&frame);
                    defer! {
                        s.pop_backtrace();
                        s.pop_pkgpath();
                    }
                    (schema.check)(s, &schema.ctx, schema_value, args, kwargs);
                }
            }
        }
    }
}

pub(crate) fn schema_with_config(
    s: &Evaluator,
    ctx: &SchemaEvalContextRef,
    schema_ctx_value: &ValueRef,
    args: &ValueRef,
    kwargs: &ValueRef,
) -> ValueRef {
    let name = { ctx.borrow().node.name.node.to_string() };
    let pkgpath = s.current_pkgpath();
    let config_keys: Vec<String> = {
        ctx.borrow()
            .config
            .as_dict_ref()
            .values
            .keys()
            .cloned()
            .collect()
    };
    let runtime_type = schema_runtime_type(&name, &pkgpath);
    // Instance package path is the last frame calling package path.
    let instance_pkgpath = s.last_pkgpath();
    // Currently, `MySchema.instances()` it is only valid for files in the main package to
    // avoid unexpected non idempotent calls. For example, I instantiated a MySchema in pkg1,
    // but the length of the list returned by calling the instances method in other packages
    // is uncertain.
    {
        let mut ctx = s.runtime_ctx.borrow_mut();
        // Record schema instance in the context
        if !ctx.instances.contains_key(&runtime_type) {
            ctx.instances
                .insert(runtime_type.clone(), IndexMap::default());
        }
        let pkg_instance_map = ctx.instances.get_mut(&runtime_type).unwrap();
        if !pkg_instance_map.contains_key(&instance_pkgpath) {
            pkg_instance_map.insert(instance_pkgpath.clone(), vec![]);
        }
        pkg_instance_map
            .get_mut(&instance_pkgpath)
            .unwrap()
            .push(schema_ctx_value.clone());
    }
    // Dict to schema
    let is_sub_schema = { ctx.borrow().is_sub_schema };
    if is_sub_schema {
        let ctx = ctx.borrow();
        // Record instance copy and convert it to schema value.
        schema_ctx_value.dict_to_schema(
            &name,
            &pkgpath,
            &config_keys,
            &ctx.config_meta,
            &ctx.optional_mapping,
            Some(args.clone()),
            Some(kwargs.clone()),
        )
    } else {
        schema_ctx_value.clone()
    }
}

fn update_schema_relaxed_attr(
    s: &Evaluator,
    ctx: &SchemaEvalContextRef,
    schema_value: &mut ValueRef,
) {
    let schema_name = { ctx.borrow().node.name.node.to_string() };
    // Do check function
    // Schema runtime index signature and relaxed check
    {
        let ctx = ctx.borrow();
        if let Some(index_signature) = &ctx.node.index_signature {
            let index_sign_value = if let Some(value) = &index_signature.node.value {
                s.walk_expr(value).expect(kcl_error::RUNTIME_ERROR_MSG)
            } else {
                s.undefined_value()
            };
            let key_name = if let Some(key_name) = &index_signature.node.key_name {
                key_name.node.as_str()
            } else {
                ""
            };
            schema_relaxed_attr_update_and_check(
                s,
                schema_value,
                &ctx.config,
                &schema_name,
                &index_sign_value,
                key_name,
                index_signature.node.key_ty.node.to_string().as_str(),
                index_signature.node.value_ty.node.to_string().as_str(),
            );
        } else {
            schema_relaxed_attr_update_and_check(
                s,
                schema_value,
                &ctx.config,
                &schema_name,
                &s.undefined_value(),
                "",
                "",
                "",
            );
        }
    }
}

fn check_schema_optional_attr(s: &Evaluator, schema_value: &ValueRef) {
    if is_top_level_schema_instance(s) {
        schema_value.schema_check_attr_optional(&mut s.runtime_ctx.borrow_mut(), true);
    }
}

/// Schema additional value check
fn schema_relaxed_attr_update_and_check(
    s: &Evaluator,
    schema_value: &mut ValueRef,
    schema_config: &ValueRef,
    schema_name: &str,
    index_sign_value: &ValueRef,
    index_key_name: &str,
    key_type: &str,
    value_type: &str,
) {
    let has_index_signature = !key_type.is_empty();
    let config = schema_config.as_dict_ref();
    for (key, value) in &config.values {
        let no_such_attr = schema_value.dict_get_value(key).is_none();
        if has_index_signature && no_such_attr {
            // Allow index signature value has different values
            // related to the index signature key name.
            let should_update =
                if let Some(index_key_value) = schema_value.dict_get_value(index_key_name) {
                    index_key_value.is_str() && key == &index_key_value.as_str()
                } else {
                    true
                };
            if should_update {
                let op = config
                    .ops
                    .get(key)
                    .unwrap_or(&ConfigEntryOperationKind::Union);
                schema_value.dict_update_entry(
                    key.as_str(),
                    &index_sign_value.deep_copy(),
                    &ConfigEntryOperationKind::Override,
                    None,
                );
                s.dict_merge_key_value_pair(
                    schema_value,
                    key.as_str(),
                    value,
                    op.clone(),
                    None,
                    false,
                );
                let value = schema_value.dict_get_value(key).unwrap();
                schema_value.dict_update_key_value(
                    key.as_str(),
                    type_pack_and_check(s, &value, vec![value_type], false),
                );
            }
        } else if !has_index_signature && no_such_attr {
            panic!("No attribute named '{key}' in the schema '{schema_name}'");
        }
    }
}

/// For a schema instance returned by the schema body. Its schema and schema expr stack
/// length are both 1, if > 1, it's not a top level schema instance.
#[inline]
fn is_top_level_schema_instance(s: &Evaluator) -> bool {
    !(s.schema_stack.borrow().len() > 1 || s.schema_expr_stack.borrow().len() > 1)
}

impl<'ctx> Evaluator<'ctx> {
    pub(crate) fn construct_schema_config_meta(
        &self,
        n: Option<&'ctx ast::Node<ast::Identifier>>,
        t: &'ctx ast::ConfigExpr,
    ) -> ValueRef {
        let mut config_meta = self.dict_value();
        if let Some(n) = n {
            let value = self.string_value(&n.filename);
            self.dict_insert_value(
                &mut config_meta,
                kclvm_runtime::CONFIG_META_FILENAME,
                &value,
            );
            let value = self.int_value(n.line as i64);
            self.dict_insert_value(&mut config_meta, kclvm_runtime::CONFIG_META_LINE, &value);
            let value = self.int_value(n.column as i64);
            self.dict_insert_value(&mut config_meta, kclvm_runtime::CONFIG_META_COLUMN, &value);
        }
        for item in &t.items {
            if let Some(key) = &item.node.key {
                let name = match &key.node {
                    ast::Expr::Identifier(t) => t.names[0].node.clone(),
                    ast::Expr::NumberLit(t) => match t.value {
                        ast::NumberLitValue::Int(i) => i.to_string(),
                        ast::NumberLitValue::Float(f) => f.to_string(),
                    },
                    ast::Expr::StringLit(t) => t.value.clone(),
                    ast::Expr::NameConstantLit(t) => match t.value {
                        ast::NameConstant::True => {
                            kclvm_runtime::KCL_NAME_CONSTANT_TRUE.to_string()
                        }
                        ast::NameConstant::False => {
                            kclvm_runtime::KCL_NAME_CONSTANT_FALSE.to_string()
                        }
                        ast::NameConstant::None => {
                            kclvm_runtime::KCL_NAME_CONSTANT_NONE.to_string()
                        }
                        ast::NameConstant::Undefined => {
                            kclvm_runtime::KCL_NAME_CONSTANT_UNDEFINED.to_string()
                        }
                    },
                    _ => format!("{:?}", key.node),
                };
                let mut config_item_meta = self.dict_value();
                let value = self.string_value(&key.filename);
                self.dict_insert_value(
                    &mut config_item_meta,
                    kclvm_runtime::CONFIG_ITEM_META_FILENAME,
                    &value,
                );
                let value = self.int_value(key.line as i64);
                self.dict_insert_value(
                    &mut config_item_meta,
                    kclvm_runtime::CONFIG_ITEM_META_LINE,
                    &value,
                );
                let value = self.int_value(key.column as i64);
                self.dict_insert_value(
                    &mut config_item_meta,
                    kclvm_runtime::CONFIG_ITEM_META_COLUMN,
                    &value,
                );
                let value = match &item.node.value.node {
                    ast::Expr::Config(config_expr) => {
                        self.construct_schema_config_meta(None, config_expr)
                    }
                    _ => self.dict_value(),
                };
                self.dict_insert_value(
                    &mut config_item_meta,
                    kclvm_runtime::CONFIG_ITEM_META,
                    &value,
                );
                self.dict_insert_value(&mut config_meta, &name, &config_item_meta)
            }
        }
        config_meta
    }

    pub(crate) fn update_schema_or_rule_scope_value(
        &self,
        name: &str,               // Schema attribute name
        value: Option<&ValueRef>, // Optional right override value
    ) {
        let (mut schema_value, config_value, _) = self
            .get_schema_or_rule_config_info()
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        let config_value = config_value
            .dict_get_entry(name)
            .unwrap_or(self.undefined_value());
        if self.scope_level() >= INNER_LEVEL && !self.is_local_var(name) {
            if let Some(value) = value {
                self.schema_dict_merge(
                    &mut schema_value,
                    name,
                    value,
                    &ast::ConfigEntryOperation::Override,
                    None,
                );
            }
            self.value_union(&mut schema_value, &config_value);
            // Set config cache for the schema eval context.
            if let Some(schema_ctx) = self.get_schema_eval_context() {
                schema_ctx.borrow().set_value(self, name);
            }
        }
    }
}
