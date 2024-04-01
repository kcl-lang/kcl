use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use indexmap::IndexMap;
use kclvm_ast::ast;
use kclvm_ast::walker::TypedResultWalker;
use kclvm_runtime::{schema_runtime_type, ConfigEntryOperationKind, ValueRef, MAIN_PKG_PATH};

use crate::lazy::LazyEvalScope;
use crate::proxy::{call_schema_body, call_schema_check};
use crate::rule::RuleEvalContext;
use crate::ty::type_pack_and_check;
use crate::{error as kcl_error, Proxy};
use crate::{Evaluator, INNER_LEVEL};

pub type SchemaBodyHandler =
    Arc<dyn Fn(&Evaluator, &SchemaEvalContextRef, &ValueRef, &ValueRef) -> ValueRef>;

pub type SchemaEvalContextRef = Rc<RefCell<SchemaEvalContext>>;

/// Proxy functions represent the saved functions of the runtime its,
/// rather than executing KCL defined functions or plugin functions.
#[derive(Clone, Debug)]
pub struct SchemaEvalContext {
    pub node: ast::SchemaStmt,
    pub scope: LazyEvalScope,
    pub value: ValueRef,
    pub config: ValueRef,
    pub config_meta: ValueRef,
    pub optional_mapping: ValueRef,
    pub is_sub_schema: bool,
    pub record_instance: bool,
}

impl SchemaEvalContext {
    #[inline]
    pub fn new_with_node(node: ast::SchemaStmt) -> Self {
        SchemaEvalContext {
            node,
            scope: LazyEvalScope::default(),
            value: ValueRef::dict(None),
            config: ValueRef::dict(None),
            config_meta: ValueRef::dict(None),
            optional_mapping: ValueRef::dict(None),
            is_sub_schema: true,
            record_instance: true,
        }
    }

    /// Reset schema evaluation context state.
    pub fn reset_with_config(&mut self, config: ValueRef, config_meta: ValueRef) {
        self.config = config;
        self.config_meta = config_meta;
        self.value = ValueRef::dict(None);
        self.optional_mapping = ValueRef::dict(None);
        self.is_sub_schema = true;
        self.record_instance = true;
    }

    /// Pass value references from other schema eval context.
    /// Note that do not change the schema node.
    pub fn set_info_with_schema(&mut self, other: &SchemaEvalContext) {
        self.config = other.config.clone();
        self.config_meta = other.config_meta.clone();
        self.value = other.value.clone();
        self.optional_mapping = other.optional_mapping.clone();
        self.record_instance = other.record_instance;
        self.is_sub_schema = false;
    }

    /// Pass value references from other rule eval context.
    /// Note that do not change the schema node.
    pub fn set_info_with_rule(&mut self, other: &RuleEvalContext) {
        self.config = other.config.clone();
        self.config_meta = other.config_meta.clone();
        self.value = other.value.clone();
        self.optional_mapping = other.optional_mapping.clone();
        self.record_instance = other.record_instance;
        // Note that for the host schema, it will evaluate the final value.
        self.is_sub_schema = true;
    }

    /// Update parent schema and mixin schema information
    pub fn get_parent_schema(
        s: &Evaluator,
        ctx: &SchemaEvalContextRef,
    ) -> Option<SchemaEvalContextRef> {
        if let Some(parent_name) = &ctx.borrow().node.parent_name {
            let func = s
                .walk_identifier_with_ctx(&parent_name.node, &ast::ExprContext::Load, None)
                .expect(kcl_error::RUNTIME_ERROR_MSG);
            if let Some(index) = func.try_get_proxy() {
                let frame = {
                    let frames = s.frames.borrow();
                    frames
                        .get(index)
                        .expect(kcl_error::INTERNAL_ERROR_MSG)
                        .clone()
                };
                if let Proxy::Schema(schema) = &frame.proxy {
                    Some(schema.ctx.clone())
                } else {
                    None
                }
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
        ctx: &SchemaEvalContextRef,
    ) -> Vec<SchemaEvalContextRef> {
        let mut results = vec![];
        for mixin in &ctx.borrow().node.mixins {
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
                if let Proxy::Schema(schema) = &frame.proxy {
                    results.push(schema.ctx.clone())
                }
            }
        }
        results
    }

    /// Whether the attribute is the schema context.
    pub fn has_attr(s: &Evaluator, ctx: &SchemaEvalContextRef, name: &str) -> bool {
        for stmt in &ctx.borrow().node.body {
            if let ast::Stmt::SchemaAttr(attr) = &stmt.node {
                if &attr.name.node == name {
                    return true;
                }
            }
        }
        if let Some(parent) = SchemaEvalContext::get_parent_schema(s, ctx) {
            return SchemaEvalContext::has_attr(s, &parent, name);
        }
        false
    }

    /// Whether the index signature is the schema context.
    pub fn has_index_signature(s: &Evaluator, ctx: &SchemaEvalContextRef) -> bool {
        if ctx.borrow().node.index_signature.is_some() {
            return true;
        }
        if let Some(parent) = SchemaEvalContext::get_parent_schema(s, ctx) {
            return SchemaEvalContext::has_index_signature(s, &parent);
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
}

#[derive(Clone, Default, Debug)]
pub struct ConfigMeta {
    pub filename: String,
    pub line: u64,
    pub column: u64,
    pub item_meta: IndexMap<String, ConfigMeta>,
}

#[derive(Clone)]
pub struct SchemaCaller {
    pub ctx: SchemaEvalContextRef,
    pub body: SchemaBodyHandler,
    pub check: SchemaBodyHandler,
}

/// Schema body function
pub(crate) fn schema_body(
    s: &Evaluator,
    ctx: &SchemaEvalContextRef,
    args: &ValueRef,
    kwargs: &ValueRef,
) -> ValueRef {
    s.push_schema();
    s.enter_scope_with_schema_eval_context(ctx);
    let schema_name = &ctx.borrow().node.name.node;
    // Evaluate arguments and keyword arguments and store values to local variables.
    s.walk_arguments(&ctx.borrow().node.args, args, kwargs);
    // Schema Value
    let mut schema_value = ctx.borrow().value.clone();
    if let Some(parent_name) = &ctx.borrow().node.parent_name {
        let base_constructor_func = s
            .walk_identifier_with_ctx(&parent_name.node, &ast::ExprContext::Load, None)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        // Call base schema function
        call_schema_body(s, &base_constructor_func, args, kwargs, ctx);
    }
    // Eval schema body and record schema instances.
    if ctx.borrow().record_instance {
        let schema_pkgpath = &s.current_pkgpath();
        // Run schema compiled function
        for stmt in &ctx.borrow().node.body {
            s.walk_stmt(stmt).expect(kcl_error::RUNTIME_ERROR_MSG);
        }
        // Schema decorators check
        for decorator in &ctx.borrow().node.decorators {
            s.walk_decorator_with_name(&decorator.node, Some(schema_name), true)
                .expect(kcl_error::RUNTIME_ERROR_MSG);
        }
        let runtime_type = kclvm_runtime::schema_runtime_type(schema_name, schema_pkgpath);
        schema_value.set_potential_schema_type(&runtime_type);
        // Set schema arguments and keyword arguments
        schema_value.set_schema_args(args, kwargs);
    }
    // Schema Mixins
    for mixin in &ctx.borrow().node.mixins {
        let mixin_func = s
            .walk_identifier_with_ctx(&mixin.node, &ast::ExprContext::Load, None)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        schema_value = call_schema_body(s, &mixin_func, args, kwargs, ctx);
    }
    // Schema Attribute optional check
    let mut optional_mapping = ctx.borrow().optional_mapping.clone();
    for stmt in &ctx.borrow().node.body {
        if let ast::Stmt::SchemaAttr(schema_attr) = &stmt.node {
            s.dict_insert_value(
                &mut optional_mapping,
                &schema_attr.name.node,
                &s.bool_value(schema_attr.is_optional),
            )
        }
    }
    // Do schema check for the sub schema.
    if ctx.borrow().is_sub_schema {
        let index_sign_key_name = if let Some(index_signature) = &ctx.borrow().node.index_signature
        {
            if let Some(key_name) = &index_signature.node.key_name {
                key_name.clone()
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };
        if index_sign_key_name.is_empty() {
            // Call schema check block function
            schema_check(s, ctx, args, kwargs);
        } else {
            // Do check function for every index signature key
            let config = ctx.borrow().config.clone();
            for (k, _) in &config.as_dict_ref().values {
                // relaxed keys
                if schema_value.attr_map_get(k).is_none() {
                    // Update index signature key value
                    let value = ValueRef::str(k);
                    schema_value.dict_update_key_value(&index_sign_key_name, value);
                    // Call schema check block function
                    schema_check(s, ctx, args, kwargs);
                }
            }
            schema_value.dict_remove(&index_sign_key_name);
        }
    }
    s.leave_scope();
    s.pop_schema();
    schema_with_config(s, ctx, &schema_value, args, kwargs)
}

pub(crate) fn schema_with_config(
    s: &Evaluator,
    ctx: &SchemaEvalContextRef,
    schema_dict: &ValueRef,
    args: &ValueRef,
    kwargs: &ValueRef,
) -> ValueRef {
    let name = ctx.borrow().node.name.node.to_string();
    let pkgpath = s.current_pkgpath();
    let config_keys: Vec<String> = ctx
        .borrow()
        .config
        .as_dict_ref()
        .values
        .keys()
        .cloned()
        .collect();
    let schema = schema_dict.dict_to_schema(
        &name,
        &pkgpath,
        &config_keys,
        &ctx.borrow().config_meta,
        &ctx.borrow().optional_mapping,
        Some(args.clone()),
        Some(kwargs.clone()),
    );
    let runtime_type = schema_runtime_type(&name, &pkgpath);
    // Instance package path is the last frame calling package path.
    let instance_pkgpath = s.last_pkgpath();
    if ctx.borrow().record_instance
        && (instance_pkgpath.is_empty() || instance_pkgpath == MAIN_PKG_PATH)
    {
        let mut ctx = s.runtime_ctx.borrow_mut();
        // Record schema instance in the context
        if !ctx.instances.contains_key(&runtime_type) {
            ctx.instances.insert(runtime_type.clone(), vec![]);
        }
        ctx.instances
            .get_mut(&runtime_type)
            .unwrap()
            .push(schema_dict.clone());
    }
    // Dict to schema
    if ctx.borrow().is_sub_schema {
        schema
    } else {
        schema_dict.clone()
    }
}

// Schema check function
pub(crate) fn schema_check(
    s: &Evaluator,
    ctx: &SchemaEvalContextRef,
    args: &ValueRef,
    kwargs: &ValueRef,
) -> ValueRef {
    let schema_name = &ctx.borrow().node.name.node;
    let mut schema_value = ctx.borrow().value.clone();
    // Do check function
    // Schema runtime index signature and relaxed check
    if let Some(index_signature) = &ctx.borrow().node.index_signature {
        let index_sign_value = if let Some(value) = &index_signature.node.value {
            s.walk_expr(value).expect(kcl_error::RUNTIME_ERROR_MSG)
        } else {
            s.undefined_value()
        };
        let key_name = if let Some(key_name) = &index_signature.node.key_name {
            key_name.as_str()
        } else {
            ""
        };
        schema_value_check(
            s,
            &mut schema_value,
            &ctx.borrow().config,
            schema_name,
            &index_sign_value,
            key_name,
            index_signature.node.key_ty.node.to_string().as_str(),
            index_signature.node.value_ty.node.to_string().as_str(),
        );
    } else {
        schema_value_check(
            s,
            &mut schema_value,
            &ctx.borrow().config,
            schema_name,
            &s.undefined_value(),
            "",
            "",
            "",
        );
    }
    // Call base check function
    if let Some(parent_name) = &ctx.borrow().node.parent_name {
        let base_constructor_func = s
            .walk_identifier_with_ctx(&parent_name.node, &ast::ExprContext::Load, None)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        call_schema_check(s, &base_constructor_func, args, kwargs, Some(ctx))
    }
    // Call self check function
    for check_expr in &ctx.borrow().node.checks {
        s.walk_check_expr(&check_expr.node)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
    }
    // Call mixin check functions
    for mixin in &ctx.borrow().node.mixins {
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
                (schema.check)(s, &schema.ctx, args, kwargs);
                s.pop_pkgpath();
            }
        }
    }
    schema_value
}

/// Schema additional value check
fn schema_value_check(
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
                    &-1,
                );
                s.dict_merge_key_value_pair(
                    schema_value,
                    key.as_str(),
                    value,
                    op.clone(),
                    -1,
                    false,
                );
                let value = schema_value.dict_get_value(key).unwrap();
                schema_value.dict_update_key_value(
                    key.as_str(),
                    type_pack_and_check(s, &value, vec![value_type]),
                );
            }
        } else if !has_index_signature && no_such_attr {
            panic!("No attribute named '{key}' in the schema '{schema_name}'");
        }
    }
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
            .unwrap_or(self.none_value());
        if self.scope_level() >= INNER_LEVEL && !self.is_local_var(name) {
            if let Some(value) = value {
                self.schema_dict_merge(
                    &mut schema_value,
                    name,
                    value,
                    &ast::ConfigEntryOperation::Override,
                    -1,
                );
            }
            self.value_union(&mut schema_value, &config_value);
        }
    }
}
