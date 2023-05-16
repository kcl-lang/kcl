use std::rc::Rc;

use super::{
    node::TypeRef,
    scope::{ScopeKind, ScopeObject, ScopeObjectKind},
    Resolver,
};
use crate::ty::sup;
use crate::ty::SchemaType;
use crate::ty::{Type, TypeKind};
use kclvm_ast::ast;
use kclvm_ast::pos::GetPos;
use kclvm_error::{ErrorKind, Message, Position, Style};

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
        ty: Rc<Type>,
        start: Position,
        end: Position,
    ) -> ScopeObject {
        ScopeObject {
            name: name.to_string(),
            start,
            end,
            ty,
            kind: ScopeObjectKind::Attribute,
            used: false,
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
                            TypeKind::Dict(_, val_ty) => Some(self.new_config_expr_context_item(
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
                                        attr_ty_obj.pos.clone(),
                                        attr_ty_obj.pos.clone(),
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
                    ast::Expr::Identifier(identifier) => identifier.names.clone(),
                    ast::Expr::Subscript(subscript) => {
                        if let ast::Expr::Identifier(identifier) = &subscript.value.node {
                            if let Some(index) = &subscript.index {
                                if matches!(index.node, ast::Expr::NumberLit(_)) {
                                    identifier.names.clone()
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
    ///     the item poped from stack.
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
                if let TypeKind::Schema(schema_ty) = &obj.ty.kind {
                    self.check_config_attr(name, &key.get_pos(), schema_ty);
                }
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
                    ast::Expr::Identifier(identifier) => identifier.names.clone(),
                    ast::Expr::Subscript(subscript) => {
                        if let ast::Expr::Identifier(identifier) = &subscript.value.node {
                            if let Some(index) = &subscript.index {
                                if matches!(index.node, ast::Expr::NumberLit(_)) {
                                    has_index = true;
                                    identifier.names.clone()
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
                    let pos = obj_last.start.clone();
                    self.must_assignable_to(val_ty, ty, key.get_pos(), Some(pos));
                }
                self.clear_config_expr_context(stack_depth, false);
            }
        }
    }

    /// Check config attr has been defined.
    pub(crate) fn check_config_attr(&mut self, attr: &str, pos: &Position, schema_ty: &SchemaType) {
        let runtime_type = kclvm_runtime::schema_runtime_type(&schema_ty.name, &schema_ty.pkgpath);
        match self.ctx.schema_mapping.get(&runtime_type) {
            Some(schema_mapping_ty) => {
                let schema_ty = schema_mapping_ty.borrow();
                if schema_ty.get_obj_of_attr(attr).is_none()
                    && !schema_ty.is_mixin
                    && schema_ty.index_signature.is_none()
                {
                    self.handler.add_compile_error(
                        &format!(
                            "Cannot add member '{}' to schema '{}'",
                            attr, schema_ty.name
                        ),
                        pos.clone(),
                    );
                }
            }
            None => {
                if schema_ty.get_obj_of_attr(attr).is_none()
                    && !schema_ty.is_mixin
                    && schema_ty.index_signature.is_none()
                {
                    self.handler.add_compile_error(
                        &format!(
                            "Cannot add member '{}' to schema '{}'",
                            attr, schema_ty.name
                        ),
                        pos.clone(),
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
    ) -> (bool, Rc<Type>) {
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
        self.enter_scope(
            self.ctx.start_pos.clone(),
            self.ctx.end_pos.clone(),
            ScopeKind::Config,
        );
        let mut key_types: Vec<TypeRef> = vec![];
        let mut val_types: Vec<TypeRef> = vec![];
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
                            let name = &identifier.names[0];
                            let key_ty = if self.ctx.local_vars.contains(name) {
                                self.expr(key)
                            } else {
                                Rc::new(Type::str_lit(name))
                            };
                            self.check_attr_ty(&key_ty, key.get_pos());
                            self.insert_object(
                                name,
                                ScopeObject {
                                    name: name.to_string(),
                                    start: key.get_pos(),
                                    end: key.get_end_pos(),
                                    ty: val_ty.clone(),
                                    kind: ScopeObjectKind::Attribute,
                                    used: false,
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
                        self.check_attr_ty(&key_ty, key.get_pos());
                        if let ast::Expr::StringLit(string_lit) = &key.node {
                            self.insert_object(
                                &string_lit.value,
                                ScopeObject {
                                    name: string_lit.value.clone(),
                                    start: key.get_pos(),
                                    end: key.get_end_pos(),
                                    ty: val_ty.clone(),
                                    kind: ScopeObjectKind::Attribute,
                                    used: false,
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
                        TypeKind::Dict(key_ty, val_ty) => {
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
                                value.get_pos(),
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
                        pos: value.get_pos(),
                        style: Style::LineAndColumn,
                        message: format!(
                            "only list type can in inserted, got '{}'",
                            val_ty.ty_str()
                        ),
                        note: None,
                    }],
                );
            }
            self.clear_config_expr_context(stack_depth, false);
        }
        self.leave_scope();
        let key_ty = sup(&key_types);
        let val_ty = sup(&val_types);
        Type::dict_ref(key_ty, val_ty)
    }
}
