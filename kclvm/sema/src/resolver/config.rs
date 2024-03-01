use std::sync::Arc;

use super::{
    scope::{ScopeKind, ScopeObject, ScopeObjectKind},
    Resolver,
};
use crate::ty::{sup, DictType, TypeInferMethods, TypeRef};
use crate::ty::{Attr, SchemaType};
use crate::ty::{Type, TypeKind};
use indexmap::IndexMap;
use kclvm_ast::ast;
use kclvm_ast::pos::GetPos;
use kclvm_error::{diagnostic::Range, ErrorKind, Message, Position, Style};

/// Config Expr type check state.
///
/// e.g.
/// ```no_check
/// schema Person:
///     name: str
///
/// person = Person {
///     name: 1  # Type error, expect str, got int(1)
/// }
/// ```
pub enum SwitchConfigContextState {
    KeepConfigUnchanged = 0,
    SwitchConfigOnce = 1,
}

impl<'ctx> Resolver<'ctx> {
    #[inline]
    pub(crate) fn new_config_expr_context_item(
        &mut self,
        name: &str,
        ty: TypeRef,
        start: Position,
        end: Position,
    ) -> ScopeObject {
        ScopeObject {
            name: name.to_string(),
            start,
            end,
            ty,
            kind: ScopeObjectKind::Attribute,
            doc: None,
        }
    }

    /// Finds the items needed to switch the context by name 'key_name'
    ///
    /// At present, only when the top item of the stack is 'KCLSchemaTypeObject' or 'KCLDictTypeObject',
    /// it will return the next item (the attribute named 'key_name' in 'KCLSchemaTypeObject'
    /// or the value of 'key_name' in 'KCLDictTypeObject') needed to be pushed.
    /// If the top item of the stack is not 'KCLSchemaTypeObject' or 'KCLDictTypeObject',
    /// it will return 'None'.
    ///
    /// Args:
    ///     key_name: The name of the item needed to be pushed onto the 'config_expr_context' stack
    ///
    /// Returns:
    ///     The item needed to be pushed onto the 'config_expr_context' stack
    pub(crate) fn find_schema_attr_obj_from_schema_expr_stack(
        &mut self,
        key_name: &str,
    ) -> Option<ScopeObject> {
        if key_name.is_empty() {
            None
        } else {
            match self.ctx.config_expr_context.last() {
                Some(obj) => {
                    let obj = obj.clone();
                    match obj {
                        Some(obj) => match &obj.ty.kind {
                            TypeKind::List(elem_type) => Some(self.new_config_expr_context_item(
                                key_name,
                                elem_type.clone(),
                                obj.start.clone(),
                                obj.end.clone(),
                            )),
                            TypeKind::Dict(DictType {
                                key_ty: _, val_ty, ..
                            }) => Some(self.new_config_expr_context_item(
                                key_name,
                                val_ty.clone(),
                                obj.start.clone(),
                                obj.end.clone(),
                            )),
                            TypeKind::Schema(schema_ty) => {
                                match schema_ty.get_obj_of_attr(key_name) {
                                    Some(attr_ty_obj) => Some(self.new_config_expr_context_item(
                                        key_name,
                                        attr_ty_obj.ty.clone(),
                                        attr_ty_obj.range.0.clone(),
                                        attr_ty_obj.range.1.clone(),
                                    )),
                                    None => match &schema_ty.index_signature {
                                        Some(index_signature) => {
                                            Some(self.new_config_expr_context_item(
                                                key_name,
                                                index_signature.val_ty.clone(),
                                                obj.start.clone(),
                                                obj.end.clone(),
                                            ))
                                        }
                                        None => None,
                                    },
                                }
                            }
                            _ => None,
                        },
                        None => None,
                    }
                }
                None => None,
            }
        }
    }

    /// Switch the context in 'config_expr_context' stack by AST nodes 'Identifier', 'Subscript' or 'Literal'
    ///
    /// Args:
    ///     key: AST nodes 'Identifier', 'Subscript' or 'Literal'
    ///
    /// Returns:
    ///     push stack times
    pub(crate) fn switch_config_expr_context_by_key(
        &mut self,
        key: &'ctx Option<ast::NodeRef<ast::Expr>>,
    ) -> usize {
        match key {
            Some(key) => {
                let names: Vec<String> = match &key.node {
                    ast::Expr::Identifier(identifier) => identifier.get_names(),
                    ast::Expr::Subscript(subscript) => {
                        if let ast::Expr::Identifier(identifier) = &subscript.value.node {
                            if let Some(index) = &subscript.index {
                                if matches!(index.node, ast::Expr::NumberLit(_)) {
                                    identifier.get_names()
                                } else {
                                    return SwitchConfigContextState::KeepConfigUnchanged as usize;
                                }
                            } else {
                                return SwitchConfigContextState::KeepConfigUnchanged as usize;
                            }
                        } else {
                            return SwitchConfigContextState::KeepConfigUnchanged as usize;
                        }
                    }
                    ast::Expr::StringLit(string_lit) => vec![string_lit.value.clone()],
                    // There may be a valid configuration key for joined string and missing expressions here,
                    // and we will restore it to a null value to avoid unfriendly error messages.
                    ast::Expr::JoinedString(_) | ast::Expr::Missing(_) => vec!["".to_string()],
                    _ => return SwitchConfigContextState::KeepConfigUnchanged as usize,
                };
                self.switch_config_expr_context_by_names(&names)
            }
            None => SwitchConfigContextState::KeepConfigUnchanged as usize,
        }
    }

    /// Switch the context in 'config_expr_context' stack by the list index `[]`
    ///
    /// Returns:
    ///     push stack times
    #[inline]
    pub(crate) fn switch_list_expr_context(&mut self) -> usize {
        self.switch_config_expr_context_by_names(&["[]".to_string()])
    }

    /// Switch the context in 'config_expr_context' stack by name
    ///
    /// find the next item that needs to be pushed into the stack,
    /// according to name and the top context of the stack, and push the item into the stack.
    ///
    /// Args:
    ///     name: the name of item to be pushed
    ///
    /// Returns:
    ///     push stack times
    pub(crate) fn switch_config_expr_context_by_name(&mut self, name: &str) -> usize {
        let ctx_obj = self.find_schema_attr_obj_from_schema_expr_stack(name);
        self.switch_config_expr_context(ctx_obj) as usize
    }

    /// Push method for the 'config_expr_context' stack
    ///
    /// Args:
    ///     config_ctx_obj: the item needed to be pushed
    ///
    /// Returns:
    ///     push stack times
    pub(crate) fn switch_config_expr_context(
        &mut self,
        config_ctx_obj: Option<ScopeObject>,
    ) -> SwitchConfigContextState {
        self.ctx.config_expr_context.push(config_ctx_obj);
        SwitchConfigContextState::SwitchConfigOnce
    }

    /// Pop method for the 'config_expr_context' stack
    ///
    /// Returns:
    ///     the item popped from stack.
    #[inline]
    pub(crate) fn restore_config_expr_context(&mut self) -> Option<ScopeObject> {
        match self.ctx.config_expr_context.pop() {
            Some(obj) => obj,
            None => None,
        }
    }

    /// Pop all method for the 'config_expr_context' stack
    ///
    /// Args:
    ///     stack_depth: 'stack_depth' is the number of stacks that need to be popped
    ///     clear_all: 'clear_all' is True to clear all the items of the stack
    ///
    pub(crate) fn clear_config_expr_context(&mut self, stack_depth: usize, clear_all: bool) {
        if clear_all {
            self.ctx.config_expr_context.clear()
        } else {
            for _ in 0..stack_depth {
                self.restore_config_expr_context();
            }
        }
    }

    /// Switch the context in 'config_expr_context' stack by names
    ///
    /// Traverse all name in 'names', find the next item that needs to be pushed into the stack,
    /// according to name and the top context of the stack, and push the item into the stack.
    ///
    /// Args:
    ///     names: A list of string containing the names of items to be pushed
    ///
    /// Returns:
    ///     push stack times
    pub(crate) fn switch_config_expr_context_by_names(&mut self, names: &[String]) -> usize {
        let mut stack_depth = 0;
        for name in names {
            stack_depth += self.switch_config_expr_context_by_name(name);
        }
        stack_depth
    }

    /// Check whether the key of config expr meets the constraints of schema attributes such as final, defined.
    ///
    /// Args:
    ///     name: the name of key
    ///     key: the ast node of key
    ///     check_rules: the constraints, such as 'check_defined'
    pub(crate) fn check_config_expr_by_key_name(
        &mut self,
        name: &str,
        key: &'ctx ast::NodeRef<ast::Expr>,
    ) {
        if !name.is_empty() {
            if let Some(Some(obj)) = self.ctx.config_expr_context.last() {
                let obj = obj.clone();
                self.must_check_config_attr(name, &key.get_span_pos(), &obj.ty);
            }
        }
    }

    /// Check the key-value in 'ConfigExpr', such as check_defined and check_type
    ///
    /// Notes:
    ///     If the top item of the 'config_expr_context' stack is 'None', the check will be skipped.
    ///
    /// Args:
    ///     key: the key of 'ConfigExpr'.
    ///     value: the value of 'ConfigExpr'.
    ///     check_rules: Some checks on the key individually，such as check_defined.
    pub(crate) fn check_config_entry(
        &mut self,
        key: &'ctx Option<ast::NodeRef<ast::Expr>>,
        value: &'ctx ast::NodeRef<ast::Expr>,
    ) {
        if let Some(key) = key {
            if let Some(Some(_)) = self.ctx.config_expr_context.last() {
                let mut has_index = false;
                let names: Vec<String> = match &key.node {
                    ast::Expr::Identifier(identifier) => identifier.get_names(),
                    ast::Expr::Subscript(subscript) => {
                        if let ast::Expr::Identifier(identifier) = &subscript.value.node {
                            if let Some(index) = &subscript.index {
                                if matches!(index.node, ast::Expr::NumberLit(_)) {
                                    has_index = true;
                                    identifier.get_names()
                                } else {
                                    return;
                                }
                            } else {
                                return;
                            }
                        } else {
                            return;
                        }
                    }
                    ast::Expr::StringLit(string_lit) => vec![string_lit.value.clone()],
                    _ => return,
                };
                let mut stack_depth = 0;
                for name in &names {
                    self.check_config_expr_by_key_name(name, key);
                    stack_depth += self.switch_config_expr_context_by_name(name);
                }
                let mut val_ty = self.expr(value);
                for _ in 0..names.len() - 1 {
                    val_ty = Type::dict_ref(self.str_ty(), val_ty);
                }
                if has_index {
                    val_ty = Type::list_ref(val_ty);
                }
                if let Some(Some(obj_last)) = self.ctx.config_expr_context.last() {
                    let ty = obj_last.ty.clone();
                    self.must_assignable_to(
                        val_ty,
                        ty,
                        key.get_span_pos(),
                        Some(obj_last.get_span_pos()),
                    );
                }
                self.clear_config_expr_context(stack_depth, false);
            }
        }
    }

    pub(crate) fn get_config_attr_err_suggestion(
        &self,
        attr: &str,
        schema_ty: &SchemaType,
    ) -> (Vec<String>, String) {
        let mut suggestion = String::new();
        // Calculate the closest miss attributes.
        let suggs = suggestions::provide_suggestions(attr, schema_ty.attrs.keys());
        if suggs.len() > 0 {
            suggestion = format!(", did you mean '{:?}'?", suggs);
        }
        (suggs, suggestion)
    }

    /// Check config attr has been defined.
    pub(crate) fn must_check_config_attr(&mut self, attr: &str, range: &Range, ty: &TypeRef) {
        if let TypeKind::Schema(schema_ty) = &ty.kind {
            self.check_config_attr(attr, range, schema_ty)
        } else if let TypeKind::Union(types) = &ty.kind {
            let mut schema_names = vec![];
            let mut total_suggs = vec![];
            for ty in types {
                if let TypeKind::Schema(schema_ty) = &ty.kind {
                    if schema_ty.get_obj_of_attr(attr).is_none()
                        && !schema_ty.is_mixin
                        && schema_ty.index_signature.is_none()
                    {
                        let mut suggs =
                            suggestions::provide_suggestions(attr, schema_ty.attrs.keys());
                        total_suggs.append(&mut suggs);
                        schema_names.push(schema_ty.name.clone());
                    } else {
                        // If there is a schema attribute that meets the condition, the type check passes
                        return;
                    }
                }
            }
            if !schema_names.is_empty() {
                self.handler.add_compile_error_with_suggestions(
                    &format!(
                        "Cannot add member '{}' to '{}'{}",
                        attr,
                        if schema_names.len() > 1 {
                            format!("schemas {:?}", schema_names)
                        } else {
                            format!("schema {}", schema_names[0])
                        },
                        if total_suggs.is_empty() {
                            "".to_string()
                        } else {
                            format!(", did you mean '{:?}'?", total_suggs)
                        },
                    ),
                    range.clone(),
                    Some(total_suggs),
                );
            }
        }
    }

    /// Check config attr has been defined.
    pub(crate) fn check_config_attr(&mut self, attr: &str, range: &Range, schema_ty: &SchemaType) {
        let runtime_type = kclvm_runtime::schema_runtime_type(&schema_ty.name, &schema_ty.pkgpath);
        match self.ctx.schema_mapping.get(&runtime_type) {
            Some(schema_mapping_ty) => {
                let schema_ty_ref = schema_mapping_ty.borrow();
                if schema_ty_ref.get_obj_of_attr(attr).is_none()
                    && !schema_ty_ref.is_mixin
                    && schema_ty_ref.index_signature.is_none()
                {
                    let (suggs, msg) = self.get_config_attr_err_suggestion(attr, schema_ty);
                    self.handler.add_compile_error_with_suggestions(
                        &format!(
                            "Cannot add member '{}' to schema '{}'{}",
                            attr, schema_ty_ref.name, msg,
                        ),
                        range.clone(),
                        Some(suggs),
                    );
                }
            }
            None => {
                if schema_ty.get_obj_of_attr(attr).is_none()
                    && !schema_ty.is_mixin
                    && schema_ty.index_signature.is_none()
                {
                    let (suggs, msg) = self.get_config_attr_err_suggestion(attr, schema_ty);
                    self.handler.add_compile_error_with_suggestions(
                        &format!(
                            "Cannot add member '{}' to schema '{}'{}",
                            attr, schema_ty.name, msg,
                        ),
                        range.clone(),
                        Some(suggs),
                    );
                }
            }
        };
    }

    /// Schema load atr
    pub(crate) fn schema_load_attr(
        &mut self,
        schema_ty: &SchemaType,
        attr: &str,
    ) -> (bool, TypeRef) {
        let runtime_type = kclvm_runtime::schema_runtime_type(&schema_ty.name, &schema_ty.pkgpath);
        match self.ctx.schema_mapping.get(&runtime_type) {
            Some(schema_mapping_ty) => {
                let schema_ty = schema_mapping_ty.borrow();
                match schema_ty.get_type_of_attr(attr) {
                    Some(ty) => (true, ty),
                    None => {
                        if schema_ty.is_mixin || schema_ty.index_signature.is_some() {
                            (true, self.any_ty())
                        } else {
                            (false, self.any_ty())
                        }
                    }
                }
            }
            None => match schema_ty.get_type_of_attr(attr) {
                Some(ty) => (true, ty),
                None => {
                    if schema_ty.is_mixin || schema_ty.index_signature.is_some() {
                        (true, self.any_ty())
                    } else {
                        (false, self.any_ty())
                    }
                }
            },
        }
    }

    pub(crate) fn walk_config_entries(
        &mut self,
        entries: &'ctx [ast::NodeRef<ast::ConfigEntry>],
    ) -> TypeRef {
        let (start, end) = match entries.len() {
            0 => (self.ctx.start_pos.clone(), self.ctx.end_pos.clone()),
            1 => entries[0].get_span_pos(),
            _ => (
                entries.first().unwrap().get_pos(),
                entries.last().unwrap().get_end_pos(),
            ),
        };
        self.enter_scope(start, end, ScopeKind::Config);
        let mut key_types: Vec<TypeRef> = vec![];
        let mut val_types: Vec<TypeRef> = vec![];
        let mut attrs: IndexMap<String, Attr> = IndexMap::new();
        for item in entries {
            let key = &item.node.key;
            let value = &item.node.value;
            let op = &item.node.operation;
            let mut stack_depth: usize = 0;
            self.check_config_entry(key, value);
            stack_depth += self.switch_config_expr_context_by_key(key);
            let mut has_insert_index = false;
            let val_ty = match key {
                Some(key) => match &key.node {
                    ast::Expr::Identifier(identifier) => {
                        let mut val_ty = self.expr(value);
                        for _ in 0..identifier.names.len() - 1 {
                            val_ty = Type::dict_ref(self.str_ty(), val_ty.clone());
                        }
                        let key_ty = if identifier.names.len() == 1 {
                            let name = &identifier.names[0].node;
                            let key_ty = if self.ctx.local_vars.contains(name) {
                                self.expr(key)
                            } else {
                                Arc::new(Type::str_lit(name))
                            };
                            self.check_attr_ty(&key_ty, key.get_span_pos());
                            let ty = if let Some(attr) = attrs.get(name) {
                                sup(&[attr.ty.clone(), val_ty.clone()])
                            } else {
                                val_ty.clone()
                            };
                            attrs.insert(
                                name.to_string(),
                                Attr {
                                    ty: self.ctx.ty_ctx.infer_to_variable_type(ty.clone()),
                                    range: key.get_span_pos(),
                                },
                            );
                            self.insert_object(
                                name,
                                ScopeObject {
                                    name: name.to_string(),
                                    start: key.get_pos(),
                                    end: key.get_end_pos(),
                                    ty,
                                    kind: ScopeObjectKind::Attribute,
                                    doc: None,
                                },
                            );
                            key_ty
                        } else {
                            self.str_ty()
                        };
                        key_types.push(key_ty);
                        val_types.push(val_ty.clone());
                        val_ty
                    }
                    ast::Expr::Subscript(subscript)
                        if matches!(subscript.value.node, ast::Expr::Identifier(_)) =>
                    {
                        has_insert_index = true;
                        let val_ty = self.expr(value);
                        key_types.push(self.str_ty());
                        val_types.push(Type::list_ref(val_ty.clone()));
                        val_ty
                    }
                    _ => {
                        let key_ty = self.expr(key);
                        let val_ty = self.expr(value);
                        self.check_attr_ty(&key_ty, key.get_span_pos());
                        if let ast::Expr::StringLit(string_lit) = &key.node {
                            let ty = if let Some(attr) = attrs.get(&string_lit.value) {
                                sup(&[attr.ty.clone(), val_ty.clone()])
                            } else {
                                val_ty.clone()
                            };
                            attrs.insert(
                                string_lit.value.clone(),
                                Attr {
                                    ty: self.ctx.ty_ctx.infer_to_variable_type(ty.clone()),
                                    range: key.get_span_pos(),
                                },
                            );
                            self.insert_object(
                                &string_lit.value,
                                ScopeObject {
                                    name: string_lit.value.clone(),
                                    start: key.get_pos(),
                                    end: key.get_end_pos(),
                                    ty,
                                    kind: ScopeObjectKind::Attribute,
                                    doc: None,
                                },
                            );
                        }
                        key_types.push(key_ty);
                        val_types.push(val_ty.clone());
                        val_ty
                    }
                },
                None => {
                    let val_ty = self.expr(value);
                    match &val_ty.kind {
                        TypeKind::None | TypeKind::Any => {
                            val_types.push(val_ty.clone());
                        }
                        TypeKind::Dict(DictType { key_ty, val_ty, .. }) => {
                            key_types.push(key_ty.clone());
                            val_types.push(val_ty.clone());
                        }
                        TypeKind::Schema(schema_ty) => {
                            key_types.push(schema_ty.key_ty());
                            val_types.push(schema_ty.val_ty());
                        }
                        TypeKind::Union(types)
                            if self
                                .ctx
                                .ty_ctx
                                .is_config_type_or_config_union_type(val_ty.clone()) =>
                        {
                            key_types.push(sup(&types
                                .iter()
                                .map(|ty| ty.config_key_ty())
                                .collect::<Vec<TypeRef>>()));
                            val_types.push(sup(&types
                                .iter()
                                .map(|ty| ty.config_val_ty())
                                .collect::<Vec<TypeRef>>()));
                        }
                        _ => {
                            self.handler.add_compile_error(
                                &format!(
                                    "only dict and schema can be used ** unpack, got '{}'",
                                    val_ty.ty_str()
                                ),
                                value.get_span_pos(),
                            );
                        }
                    }
                    val_ty
                }
            };
            if matches!(op, ast::ConfigEntryOperation::Insert)
                && !has_insert_index
                && !val_ty.is_any()
                && !val_ty.is_list()
            {
                self.handler.add_error(
                    ErrorKind::IllegalAttributeError,
                    &[Message {
                        range: value.get_span_pos(),
                        style: Style::LineAndColumn,
                        message: format!(
                            "only list type can in inserted, got '{}'",
                            val_ty.ty_str()
                        ),
                        note: None,
                        suggested_replacement: None,
                    }],
                );
            }
            self.clear_config_expr_context(stack_depth, false);
        }
        self.leave_scope();
        let key_ty = sup(&key_types);
        let val_ty = sup(&val_types);
        Type::dict_ref_with_attrs(key_ty, val_ty, attrs)
    }
}
