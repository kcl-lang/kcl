use anyhow::anyhow;
use std::sync::Arc;

use indexmap::IndexMap;
use kclvm_ast::ast;
use kclvm_ast::pos::GetPos;
use kclvm_ast::walker::MutSelfTypedResultWalker;
use kclvm_error::{diagnostic::Range, Position};

use crate::{
    core::{
        scope::{ConfigScopeContext, LocalSymbolScopeKind},
        symbol::{
            CommentOrDocSymbol, DecoratorSymbol, ExpressionSymbol, Symbol, SymbolHint,
            SymbolHintKind, SymbolRef, SymbolSemanticInfo, UnresolvedSymbol, ValueSymbol,
        },
    },
    ty::{Parameter, Type, TypeKind, SCHEMA_MEMBER_FUNCTIONS},
};

use super::AdvancedResolver;

type ResolvedResult = anyhow::Result<Option<SymbolRef>>;

impl<'ctx> MutSelfTypedResultWalker<'ctx> for AdvancedResolver<'ctx> {
    type Result = anyhow::Result<Option<SymbolRef>>;

    fn walk_module(&mut self, module: &'ctx ast::Module) -> Self::Result {
        for stmt in module.body.iter() {
            self.stmt(&stmt)?;
        }
        for comment in module.comments.iter() {
            let (start, end) = comment.get_span_pos();
            self.ctx.start_pos = start;
            self.ctx.end_pos = end;
            self.ctx.cur_node = comment.id.clone();
            self.walk_comment(&comment.node)?;
        }
        Ok(None)
    }

    fn walk_expr_stmt(&mut self, expr_stmt: &'ctx ast::ExprStmt) -> Self::Result {
        for expr in expr_stmt.exprs.iter() {
            self.expr(&expr)?;
        }
        Ok(None)
    }

    fn walk_unification_stmt(
        &mut self,
        unification_stmt: &'ctx ast::UnificationStmt,
    ) -> Self::Result {
        self.ctx.maybe_def = true;
        self.walk_identifier_expr(&unification_stmt.target)?;
        // Set schema attribute if it is in the schema stmt.
        if let Some(parent_scope) = self.ctx.scopes.last() {
            if let Some(parent_scope) = self.gs.get_scopes().get_scope(&parent_scope) {
                let mut doc = None;
                if let Some(schema_symbol) = parent_scope.get_owner() {
                    let schema_symbol = self
                        .gs
                        .get_symbols()
                        .get_symbol(schema_symbol)
                        .ok_or(anyhow!("schema_symbol not found"))?;
                    if let Some(schema_ty) = schema_symbol.get_sema_info().ty.clone() {
                        if !unification_stmt.target.node.names.is_empty() {
                            let schema_ty = schema_ty.into_schema_type();
                            if let Some(attr) = schema_ty
                                .attrs
                                .get(&unification_stmt.target.node.names[0].node)
                            {
                                doc = attr.doc.clone()
                            }
                            let attr_symbol = self
                                .gs
                                .get_symbols()
                                .symbols_info
                                .node_symbol_map
                                .get(
                                    &self
                                        .ctx
                                        .get_node_key(&unification_stmt.target.node.names[0].id),
                                )
                                .cloned();
                            if let Some(attr_symbol) = attr_symbol {
                                if let Some(symbol) = self
                                    .gs
                                    .get_symbols_mut()
                                    .attributes
                                    .get_mut(attr_symbol.get_id())
                                {
                                    symbol.sema_info = SymbolSemanticInfo {
                                        ty: self
                                            .ctx
                                            .node_ty_map
                                            .borrow()
                                            .get(&self.ctx.get_node_key(
                                                &unification_stmt.target.node.names[0].id,
                                            ))
                                            .map(|ty| ty.clone()),
                                        doc,
                                    };
                                }
                            }
                        }
                    }
                };
            }
        }
        self.ctx.maybe_def = false;
        self.walk_schema_expr(&unification_stmt.value.node)?;
        Ok(None)
    }

    fn walk_type_alias_stmt(&mut self, type_alias_stmt: &'ctx ast::TypeAliasStmt) -> Self::Result {
        let alias_symbol = self
            .gs
            .get_symbols()
            .get_symbol_by_fully_qualified_name(
                &(self.ctx.current_pkgpath.as_ref().unwrap().clone()
                    + "."
                    + &type_alias_stmt.type_name.node.get_name()),
            )
            .ok_or(anyhow!("alias_symbol not found"))?;
        if let Some(symbol) = self
            .gs
            .get_symbols_mut()
            .type_aliases
            .get_mut(alias_symbol.get_id())
        {
            symbol.sema_info = SymbolSemanticInfo {
                ty: self
                    .ctx
                    .node_ty_map
                    .borrow()
                    .get(&self.ctx.get_node_key(&type_alias_stmt.type_name.id))
                    .map(|ty| ty.clone()),
                doc: None,
            };
        }
        self.walk_type_expr(Some(&type_alias_stmt.ty))?;
        Ok(None)
    }

    fn walk_assign_stmt(&mut self, assign_stmt: &'ctx ast::AssignStmt) -> Self::Result {
        for target in &assign_stmt.targets {
            self.ctx.maybe_def = true;
            self.walk_target_expr_with_hint(target, assign_stmt.ty.is_none())?;
            self.ctx.maybe_def = false;
        }
        self.walk_type_expr(assign_stmt.ty.as_ref().map(|ty| ty.as_ref()))?;
        self.expr(&assign_stmt.value)?;
        Ok(None)
    }

    fn walk_aug_assign_stmt(&mut self, aug_assign_stmt: &'ctx ast::AugAssignStmt) -> Self::Result {
        self.ctx.maybe_def = true;
        self.walk_target_expr(&aug_assign_stmt.target)?;
        self.ctx.maybe_def = false;
        self.expr(&aug_assign_stmt.value)?;
        Ok(None)
    }

    fn walk_assert_stmt(&mut self, assert_stmt: &'ctx ast::AssertStmt) -> Self::Result {
        self.expr(&assert_stmt.test)?;
        if let Some(if_cond) = &assert_stmt.if_cond {
            self.expr(if_cond)?;
        }
        if let Some(msg) = &assert_stmt.msg {
            self.expr(msg)?;
        }
        Ok(None)
    }

    fn walk_if_stmt(&mut self, if_stmt: &'ctx ast::IfStmt) -> Self::Result {
        self.expr(&if_stmt.cond)?;
        for stmt in if_stmt.body.iter() {
            self.stmt(stmt)?;
        }
        for stmt in if_stmt.orelse.iter() {
            self.stmt(stmt)?;
        }
        Ok(None)
    }

    fn walk_import_stmt(&mut self, import_stmt: &'ctx ast::ImportStmt) -> Self::Result {
        let ast_id = self.ctx.cur_node.clone();
        let (start_pos, end_pos) = import_stmt
            .asname
            .clone()
            .unwrap_or(import_stmt.path.clone())
            .get_span_pos();

        let unresolved = UnresolvedSymbol::new(
            import_stmt.path.node.clone(),
            start_pos,
            end_pos,
            None,
            self.ctx.is_type_expr,
        );
        let package_symbol = match self
            .gs
            .get_symbols()
            .get_symbol_by_fully_qualified_name(&import_stmt.path.node)
        {
            Some(symbol) => symbol,
            None => return Ok(None),
        };
        let unresolved_ref = self.gs.get_symbols_mut().alloc_unresolved_symbol(
            unresolved,
            self.ctx.get_node_key(&ast_id),
            self.ctx.current_pkgpath.clone().unwrap(),
        );
        self.gs
            .get_symbols_mut()
            .set_def_and_ref(package_symbol, unresolved_ref);
        self.gs
            .get_symbols_mut()
            .symbols_info
            .node_symbol_map
            .insert(self.ctx.get_node_key(&ast_id), unresolved_ref);
        let cur_scope = *self.ctx.scopes.last().unwrap();
        self.gs
            .get_scopes_mut()
            .add_ref_to_scope(cur_scope, unresolved_ref);
        Ok(Some(unresolved_ref))
    }

    fn walk_schema_stmt(&mut self, schema_stmt: &'ctx ast::SchemaStmt) -> Self::Result {
        let (start, end) = (self.ctx.start_pos.clone(), self.ctx.end_pos.clone());
        let schema_ty = self
            .ctx
            .node_ty_map
            .borrow()
            .get(&self.ctx.get_node_key(&schema_stmt.name.id))
            .ok_or(anyhow!("schema_ty not found"))?
            .clone();
        let schema_symbol = self
            .gs
            .get_symbols()
            .get_type_symbol(&schema_ty, self.get_current_module_info())
            .ok_or(anyhow!("schema_symbol not found"))?;
        if self
            .gs
            .get_symbols()
            .schemas
            .contains(schema_symbol.get_id())
        {
            let mut schema_builtin_member = IndexMap::new();
            for name in SCHEMA_MEMBER_FUNCTIONS.iter() {
                let func_ty = Arc::new(Type::function(
                    Some(schema_ty.clone()),
                    Type::list_ref(Type::any_ref()),
                    &[],
                    "",
                    false,
                    None,
                ));
                let mut func_value = ValueSymbol::new(
                    name.to_string(),
                    Position::dummy_pos(),
                    Position::dummy_pos(),
                    Some(schema_symbol),
                    false,
                );
                func_value.sema_info.ty = Some(func_ty);
                let func_symbol_ref = self.gs.get_symbols_mut().alloc_value_symbol(
                    func_value,
                    self.ctx.get_node_key(&ast::AstIndex::default()),
                    self.ctx.current_pkgpath.clone().unwrap(),
                );
                schema_builtin_member.insert(name.to_string(), func_symbol_ref);
            }
            self.gs
                .get_symbols_mut()
                .symbols_info
                .schema_builtin_symbols
                .insert(schema_symbol, schema_builtin_member);
            self.gs
                .get_symbols_mut()
                .schemas
                .get_mut(schema_symbol.get_id())
                .ok_or(anyhow!("schema_symbol not found"))?
                .sema_info = SymbolSemanticInfo {
                ty: Some(schema_ty.clone()),
                doc: schema_stmt.doc.as_ref().map(|doc| doc.node.clone()),
            };
        }

        self.resolve_decorator(&schema_stmt.decorators);

        let mut last_end_pos = start.clone();

        self.enter_schema_def_scope(
            &schema_ty.into_schema_type().name,
            &self.ctx.current_filename.clone().unwrap(),
            start,
            end.clone(),
            LocalSymbolScopeKind::SchemaDef,
        );
        let cur_scope = *self.ctx.scopes.last().unwrap();
        self.gs
            .get_scopes_mut()
            .set_owner_to_scope(cur_scope, schema_symbol);
        if let Some(parent) = &schema_stmt.parent_name {
            self.gs
                .get_symbols_mut()
                .schemas
                .get_mut(schema_symbol.get_id())
                .ok_or(anyhow!("schema_symbol not found"))?
                .parent_schema = self.walk_identifier_expr(parent)?;
        }
        if let Some(for_host) = &schema_stmt.for_host_name {
            self.gs
                .get_symbols_mut()
                .schemas
                .get_mut(schema_symbol.get_id())
                .ok_or(anyhow!("schema_symbol not found"))?
                .for_host = self.walk_identifier_expr(for_host)?;
        }
        let mut mixins = vec![];
        for mixin in schema_stmt.mixins.iter() {
            if let Some(mixin) = self.walk_identifier_expr(mixin)? {
                mixins.push(mixin);
            }
            last_end_pos = mixin.get_end_pos();
        }
        self.gs
            .get_symbols_mut()
            .schemas
            .get_mut(schema_symbol.get_id())
            .ok_or(anyhow!("schema_symbol not found"))?
            .mixins = mixins;

        if let Some(args) = &schema_stmt.args {
            self.walk_arguments(&args.node)?;
            last_end_pos = args.get_end_pos();
        }
        if let Some(index_signature) = &schema_stmt.index_signature {
            if let Some(key_name) = &index_signature.node.key_name {
                let (start, end) = key_name.get_span_pos();
                let value = self.gs.get_symbols_mut().alloc_value_symbol(
                    ValueSymbol::new(
                        key_name.node.clone(),
                        start,
                        end,
                        Some(schema_symbol),
                        false,
                    ),
                    self.ctx.get_node_key(&index_signature.id),
                    self.ctx.current_pkgpath.clone().unwrap(),
                );
                if let Some(symbol) = self.gs.get_symbols_mut().values.get_mut(value.get_id()) {
                    symbol.sema_info = SymbolSemanticInfo {
                        ty: self
                            .ctx
                            .node_ty_map
                            .borrow()
                            .get(&self.ctx.get_node_key(&index_signature.id))
                            .map(|ty| ty.clone()),
                        doc: None,
                    };
                }

                self.gs
                    .get_scopes_mut()
                    .add_def_to_scope(cur_scope, key_name.node.clone(), value);
            }
            self.walk_type_expr(Some(&index_signature.node.value_ty))?;
            if let Some(value) = &index_signature.node.value {
                self.expr(value)?;
            };
            last_end_pos = index_signature.get_end_pos();
        }

        if let Some(doc) = &schema_stmt.doc {
            let (start, end) = doc.get_span_pos();
            let comment_symbol = CommentOrDocSymbol::new(start, end, doc.node.clone());
            self.gs.get_symbols_mut().alloc_comment_symbol(
                comment_symbol,
                self.ctx.get_node_key(&self.ctx.cur_node),
                self.ctx.current_pkgpath.clone().unwrap(),
            );
        }

        for stmt in schema_stmt.body.iter() {
            if let Some(attribute_symbol) = self.stmt(&stmt)? {
                let name = self
                    .gs
                    .get_symbols()
                    .get_symbol(attribute_symbol)
                    .ok_or(anyhow!("attribute_symbol not found"))?
                    .get_name();
                self.gs
                    .get_symbols_mut()
                    .schemas
                    .get_mut(schema_symbol.get_id())
                    .ok_or(anyhow!("schema_symbol not found"))?
                    .attributes
                    .insert(name, attribute_symbol);
            }
            last_end_pos = stmt.get_end_pos();
        }

        let has_check = !schema_stmt.checks.is_empty();
        if has_check {
            self.enter_local_scope(
                &self.ctx.current_filename.clone().unwrap(),
                last_end_pos,
                end,
                LocalSymbolScopeKind::Check,
            );
        }

        for check_expr in schema_stmt.checks.iter() {
            self.walk_check_expr(&check_expr.node)?;
        }

        if has_check {
            self.leave_scope();
        }
        self.leave_scope();

        Ok(Some(schema_symbol))
    }

    fn walk_rule_stmt(&mut self, rule_stmt: &'ctx ast::RuleStmt) -> Self::Result {
        let rule_ty = self
            .ctx
            .node_ty_map
            .borrow()
            .get(&self.ctx.get_node_key(&rule_stmt.name.id))
            .ok_or(anyhow!("rule_ty not found"))?
            .clone();
        let rule_symbol = self
            .gs
            .get_symbols()
            .get_type_symbol(&rule_ty, self.get_current_module_info())
            .ok_or(anyhow!("rule_symbol not found"))?;
        if let Some(symbol) = self
            .gs
            .get_symbols_mut()
            .rules
            .get_mut(rule_symbol.get_id())
        {
            symbol.sema_info = SymbolSemanticInfo {
                ty: self
                    .ctx
                    .node_ty_map
                    .borrow()
                    .get(&self.ctx.get_node_key(&rule_stmt.name.id))
                    .map(|ty| ty.clone()),
                doc: rule_stmt.doc.as_ref().map(|doc| doc.node.clone()),
            };
        }

        if let Some(for_host) = &rule_stmt.for_host_name {
            self.gs
                .get_symbols_mut()
                .rules
                .get_mut(rule_symbol.get_id())
                .ok_or(anyhow!("rule_symbol not found"))?
                .for_host = self.walk_identifier_expr(for_host)?;
        }
        let mut parent_rules = vec![];
        for parent_rule in rule_stmt.parent_rules.iter() {
            parent_rules.push(
                self.walk_identifier_expr(parent_rule)?
                    .ok_or(anyhow!("parent_rule not found"))?,
            );
        }
        self.gs
            .get_symbols_mut()
            .rules
            .get_mut(rule_symbol.get_id())
            .ok_or(anyhow!("rule_symbol not found"))?
            .parent_rules = parent_rules;
        self.resolve_decorator(&rule_stmt.decorators);
        Ok(Some(rule_symbol))
    }

    fn walk_quant_expr(&mut self, quant_expr: &'ctx ast::QuantExpr) -> Self::Result {
        let (start, end) = (self.ctx.start_pos.clone(), self.ctx.end_pos.clone());
        self.expr(&quant_expr.target)?;
        self.enter_local_scope(
            &self.ctx.current_filename.as_ref().unwrap().clone(),
            start,
            end,
            LocalSymbolScopeKind::Quant,
        );
        let cur_scope = *self.ctx.scopes.last().unwrap();
        for target in quant_expr.variables.iter() {
            if target.node.names.is_empty() {
                continue;
            }
            let name = target.node.get_name();
            let (start_pos, end_pos): Range = target.get_span_pos();
            let ast_id = if target.node.names.is_empty() {
                &target.id
            } else {
                &target.node.names.last().unwrap().id
            };
            let value = self.gs.get_symbols_mut().alloc_value_symbol(
                ValueSymbol::new(name.clone(), start_pos, end_pos, None, false),
                self.ctx.get_node_key(&ast_id),
                self.ctx.current_pkgpath.clone().unwrap(),
            );
            self.gs
                .get_scopes_mut()
                .add_def_to_scope(cur_scope, name, value);
            if let Some(symbol) = self.gs.get_symbols_mut().values.get_mut(value.get_id()) {
                let ty = self
                    .ctx
                    .node_ty_map
                    .borrow()
                    .get(&self.ctx.get_node_key(ast_id))
                    .map(|ty| ty.clone());
                symbol.hint = ty.as_ref().map(|ty| SymbolHint {
                    kind: SymbolHintKind::TypeHint(ty.ty_hint()),
                    pos: symbol.get_range().1,
                });
                symbol.sema_info = SymbolSemanticInfo { ty, doc: None };
            }
        }

        if let Some(if_cond) = &quant_expr.if_cond {
            self.expr(&if_cond)?;
        }
        self.expr(&quant_expr.test)?;
        self.leave_scope();
        Ok(None)
    }

    fn walk_schema_attr(&mut self, schema_attr: &'ctx ast::SchemaAttr) -> Self::Result {
        let attr_symbol = *self
            .gs
            .get_symbols()
            .symbols_info
            .node_symbol_map
            .get(&self.ctx.get_node_key(&schema_attr.name.id))
            .ok_or(anyhow!("attr_symbol not found"))?;
        let parent_scope = *self.ctx.scopes.last().unwrap();
        let parent_scope = self.gs.get_scopes().get_scope(&parent_scope).unwrap();
        let mut doc = None;
        if let Some(schema_symbol) = parent_scope.get_owner() {
            let schema_symbol = self
                .gs
                .get_symbols()
                .get_symbol(schema_symbol)
                .ok_or(anyhow!("schema_symbol not found"))?;
            if let Some(schema_ty) = schema_symbol.get_sema_info().ty.clone() {
                let schema_ty = schema_ty.into_schema_type();
                if let Some(attr) = schema_ty.attrs.get(&schema_attr.name.node) {
                    doc = attr.doc.clone()
                }
            }
        };

        if let Some(symbol) = self
            .gs
            .get_symbols_mut()
            .attributes
            .get_mut(attr_symbol.get_id())
        {
            symbol.sema_info = SymbolSemanticInfo {
                ty: self
                    .ctx
                    .node_ty_map
                    .borrow()
                    .get(&self.ctx.get_node_key(&schema_attr.name.id))
                    .map(|ty| ty.clone()),
                doc,
            };
        };

        self.walk_type_expr(Some(&schema_attr.ty))?;
        if let Some(value) = &schema_attr.value {
            self.expr(value)?;
        }

        self.resolve_decorator(&schema_attr.decorators);
        let cur_scope = *self.ctx.scopes.last().unwrap();
        let name = self
            .gs
            .get_symbols()
            .get_symbol(attr_symbol)
            .ok_or(anyhow!("attribute_symbol not found"))?
            .get_name();
        self.gs
            .get_scopes_mut()
            .add_def_to_scope(cur_scope, name, attr_symbol);
        Ok(Some(attr_symbol))
    }

    /// <body> if <cond> else <orelse> -> sup([body, orelse])
    fn walk_if_expr(&mut self, if_expr: &'ctx ast::IfExpr) -> Self::Result {
        self.expr(&if_expr.cond)?;
        self.expr(&if_expr.body)?;
        self.expr(&if_expr.orelse)?;
        Ok(None)
    }

    fn walk_unary_expr(&mut self, unary_expr: &'ctx ast::UnaryExpr) -> Self::Result {
        self.expr(&unary_expr.operand)?;
        Ok(None)
    }

    fn walk_binary_expr(&mut self, binary_expr: &'ctx ast::BinaryExpr) -> Self::Result {
        self.expr(&binary_expr.left)?;
        self.expr(&binary_expr.right)?;
        Ok(None)
    }

    fn walk_selector_expr(&mut self, selector_expr: &'ctx ast::SelectorExpr) -> Self::Result {
        self.expr(&selector_expr.value)?;
        let mut parent_ty = match self
            .ctx
            .node_ty_map
            .borrow()
            .get(&self.ctx.get_node_key(&selector_expr.value.id))
        {
            Some(ty) => ty.clone(),
            None => return Ok(None),
        };
        for name in &selector_expr.attr.node.names {
            let def_symbol_ref = match self.gs.get_symbols().get_type_attribute(
                &parent_ty,
                &name.node,
                self.get_current_module_info(),
            ) {
                Some(symbol) => symbol,
                None => return Ok(None),
            };

            let (start_pos, end_pos): Range = name.get_span_pos();
            let ast_id = name.id.clone();
            let unresolved = UnresolvedSymbol::new(
                name.node.clone(),
                start_pos,
                end_pos,
                None,
                self.ctx.is_type_expr,
            );
            let unresolved_ref = self.gs.get_symbols_mut().alloc_unresolved_symbol(
                unresolved,
                self.ctx.get_node_key(&ast_id),
                self.ctx.current_pkgpath.clone().unwrap(),
            );

            self.gs
                .get_symbols_mut()
                .set_def_and_ref(def_symbol_ref, unresolved_ref);

            let cur_scope = *self.ctx.scopes.last().unwrap();
            self.gs
                .get_scopes_mut()
                .add_ref_to_scope(cur_scope, unresolved_ref);

            parent_ty = match self
                .ctx
                .node_ty_map
                .borrow()
                .get(&self.ctx.get_node_key(&name.id))
            {
                Some(ty) => ty.clone(),
                None => return Ok(None),
            };
        }
        Ok(None)
    }

    fn walk_call_expr(&mut self, call_expr: &'ctx ast::CallExpr) -> Self::Result {
        let start = call_expr.func.get_end_pos();
        let end = self.ctx.end_pos.clone();
        let func_symbol = self.expr(&call_expr.func)?;
        let call_ty = self
            .ctx
            .node_ty_map
            .borrow()
            .get(&self.ctx.get_node_key(&call_expr.func.id))
            .map(|ty| ty.clone());

        if let Some(ty) = call_ty {
            match &ty.kind {
                TypeKind::Schema(schema_ty) => {
                    if !schema_ty.is_instance {
                        self.enter_local_scope(
                            &self.ctx.current_filename.as_ref().unwrap().clone(),
                            start,
                            end,
                            LocalSymbolScopeKind::Config,
                        );

                        if let Some(owner) = func_symbol {
                            let cur_scope = self.ctx.scopes.last().unwrap();
                            self.gs
                                .get_scopes_mut()
                                .set_owner_to_scope(*cur_scope, owner);
                        }
                        self.do_arguments_symbol_resolve_with_hint(
                            &call_expr.args,
                            &call_expr.keywords,
                            &schema_ty.func.params,
                            true,
                        )?;

                        self.leave_scope();
                    }
                }
                TypeKind::Function(func_ty) => {
                    self.enter_local_scope(
                        &self.ctx.current_filename.as_ref().unwrap().clone(),
                        start,
                        end,
                        LocalSymbolScopeKind::Callable,
                    );

                    if let Some(owner) = func_symbol {
                        let cur_scope = self.ctx.scopes.last().unwrap();
                        self.gs
                            .get_scopes_mut()
                            .set_owner_to_scope(*cur_scope, owner);
                    }

                    self.do_arguments_symbol_resolve_with_hint(
                        &call_expr.args,
                        &call_expr.keywords,
                        &func_ty.params,
                        true,
                    )?;
                    self.leave_scope();
                }
                _ => {}
            }
        }

        Ok(None)
    }

    fn walk_subscript(&mut self, subscript: &'ctx ast::Subscript) -> Self::Result {
        self.expr(&subscript.value)?;
        if let Some(index) = &subscript.index {
            self.expr(index)?;
        } else {
            for expr in [&subscript.lower, &subscript.upper, &subscript.step]
                .iter()
                .copied()
                .flatten()
            {
                self.expr(expr)?;
            }
        }
        Ok(None)
    }

    fn walk_paren_expr(&mut self, paren_expr: &'ctx ast::ParenExpr) -> Self::Result {
        self.expr(&paren_expr.expr)?;
        Ok(None)
    }

    fn walk_list_expr(&mut self, list_expr: &'ctx ast::ListExpr) -> Self::Result {
        for expr in list_expr.elts.iter() {
            self.expr(expr)?;
        }
        Ok(None)
    }

    fn walk_list_comp(&mut self, list_comp: &'ctx ast::ListComp) -> Self::Result {
        let start = list_comp.elt.get_pos();
        let end = match list_comp.generators.last() {
            Some(last) => last.get_end_pos(),
            None => list_comp.elt.get_end_pos(),
        };
        self.enter_local_scope(
            &self.ctx.current_filename.clone().unwrap(),
            start,
            end,
            LocalSymbolScopeKind::List,
        );
        for comp_clause in &list_comp.generators {
            self.walk_comp_clause(&comp_clause.node)?;
        }
        self.expr(&list_comp.elt)?;
        self.leave_scope();
        Ok(None)
    }

    fn walk_dict_comp(&mut self, dict_comp: &'ctx ast::DictComp) -> Self::Result {
        let (start, key) = match dict_comp.entry.key.as_ref() {
            Some(key) => (key.get_pos(), Some(key)),
            None => (dict_comp.entry.value.get_pos(), None),
        };

        let end = match dict_comp.generators.last() {
            Some(last) => last.get_end_pos(),
            None => dict_comp.entry.value.get_end_pos(),
        };
        self.enter_local_scope(
            &self.ctx.current_filename.clone().unwrap(),
            start,
            end,
            LocalSymbolScopeKind::Dict,
        );
        for comp_clause in &dict_comp.generators {
            self.walk_comp_clause(&comp_clause.node)?;
        }
        if let Some(key) = key {
            self.expr(key)?;
        }
        self.expr(&dict_comp.entry.value)?;
        self.leave_scope();
        Ok(None)
    }

    fn walk_list_if_item_expr(
        &mut self,
        list_if_item_expr: &'ctx ast::ListIfItemExpr,
    ) -> Self::Result {
        self.expr(&list_if_item_expr.if_cond)?;
        if let Some(orelse) = &list_if_item_expr.orelse {
            self.expr(orelse)?;
        }
        for expr in list_if_item_expr.exprs.iter() {
            self.expr(expr)?;
        }
        Ok(None)
    }

    fn walk_starred_expr(&mut self, starred_expr: &'ctx ast::StarredExpr) -> Self::Result {
        self.expr(&starred_expr.value)?;
        Ok(None)
    }

    fn walk_config_if_entry_expr(
        &mut self,
        config_if_entry_expr: &'ctx ast::ConfigIfEntryExpr,
    ) -> Self::Result {
        self.expr(&config_if_entry_expr.if_cond)?;
        self.walk_config_entries(&config_if_entry_expr.items)?;
        if let Some(expr) = config_if_entry_expr.orelse.as_ref() {
            self.expr(expr)?;
        }
        Ok(None)
    }

    fn walk_comp_clause(&mut self, comp_clause: &'ctx ast::CompClause) -> Self::Result {
        self.expr(&comp_clause.iter)?;
        for target in comp_clause.targets.iter() {
            self.ctx.maybe_def = true;
            self.walk_identifier_expr_with_hint(target, true)?;
            self.ctx.maybe_def = false;
        }
        for if_expr in comp_clause.ifs.iter() {
            self.expr(if_expr)?;
        }
        Ok(None)
    }

    fn walk_schema_expr(&mut self, schema_expr: &'ctx ast::SchemaExpr) -> Self::Result {
        self.walk_identifier_expr(&schema_expr.name)?;
        let schema_ty = self
            .ctx
            .node_ty_map
            .borrow()
            .get(&self.ctx.get_node_key(&schema_expr.name.id))
            .ok_or(anyhow!("schema_ty not found"))?
            .clone();
        match schema_ty.kind {
            TypeKind::Schema(_) => {
                self.expr(&schema_expr.config)?;
                self.do_arguments_symbol_resolve(&schema_expr.args, &schema_expr.kwargs)?;
            }
            _ => {
                // Invalid schema type, nothing todo
            }
        }
        Ok(None)
    }

    fn walk_config_expr(&mut self, config_expr: &'ctx ast::ConfigExpr) -> Self::Result {
        self.walk_config_entries(&config_expr.items)?;
        Ok(None)
    }

    fn walk_check_expr(&mut self, check_expr: &'ctx ast::CheckExpr) -> Self::Result {
        if let Some(msg) = &check_expr.msg {
            self.expr(msg)?;
        }
        if let Some(if_cond) = &check_expr.if_cond {
            self.expr(if_cond)?;
        }
        self.expr(&check_expr.test)?;
        Ok(None)
    }

    fn walk_lambda_expr(&mut self, lambda_expr: &'ctx ast::LambdaExpr) -> Self::Result {
        let (start, end) = (self.ctx.start_pos.clone(), self.ctx.end_pos.clone());
        self.enter_local_scope(
            &self.ctx.current_filename.clone().unwrap(),
            start,
            end,
            LocalSymbolScopeKind::Lambda,
        );
        if let Some(args) = &lambda_expr.args {
            self.walk_arguments(&args.node)?;
        }
        if let Some(ret_annotation_ty) = &lambda_expr.return_ty {
            self.walk_type_expr(Some(&ret_annotation_ty))?;
        }
        for stmt in lambda_expr.body.iter() {
            self.stmt(&stmt)?;
        }
        self.leave_scope();
        Ok(None)
    }

    fn walk_keyword(&mut self, keyword: &'ctx ast::Keyword) -> Self::Result {
        self.ctx.maybe_def = true;
        self.walk_identifier_expr(&keyword.arg)?;
        self.ctx.maybe_def = false;
        if let Some(value) = &keyword.value {
            self.expr(&value)?;
        }
        Ok(None)
    }

    fn walk_arguments(&mut self, arguments: &'ctx ast::Arguments) -> Self::Result {
        for (i, arg) in arguments.args.iter().enumerate() {
            let ty = arguments.get_arg_type_node(i);
            self.walk_type_expr(ty)?;
            self.ctx.maybe_def = true;
            self.walk_identifier_expr(arg)?;
            self.ctx.maybe_def = false;

            if let Some(val) = &arguments.defaults[i] {
                self.expr(val)?;
            }
        }
        Ok(None)
    }

    fn walk_compare(&mut self, compare: &'ctx ast::Compare) -> Self::Result {
        self.expr(&compare.left)?;
        for comparator in compare.comparators.iter() {
            self.expr(&comparator)?;
        }
        Ok(None)
    }

    fn walk_identifier(&mut self, identifier: &'ctx ast::Identifier) -> Self::Result {
        let symbol_ref = self.resolve_names(&identifier.names, self.ctx.maybe_def)?;
        Ok(symbol_ref)
    }

    fn walk_target(&mut self, target: &'ctx ast::Target) -> Self::Result {
        let symbol_ref = self.resolve_target(&target)?;
        Ok(symbol_ref)
    }

    fn walk_number_lit(&mut self, _number_lit: &'ctx ast::NumberLit) -> Self::Result {
        Ok(None)
    }

    fn walk_string_lit(&mut self, _string_lit: &'ctx ast::StringLit) -> Self::Result {
        Ok(None)
    }

    fn walk_name_constant_lit(
        &mut self,
        _name_constant_lit: &'ctx ast::NameConstantLit,
    ) -> Self::Result {
        Ok(None)
    }

    fn walk_joined_string(&mut self, joined_string: &'ctx ast::JoinedString) -> Self::Result {
        self.ctx.maybe_def = false;
        for expr in joined_string.values.iter() {
            self.expr(expr)?;
        }
        Ok(None)
    }

    fn walk_formatted_value(&mut self, formatted_value: &'ctx ast::FormattedValue) -> Self::Result {
        self.expr(&formatted_value.value)?;
        Ok(None)
    }

    fn walk_comment(&mut self, comment: &'ctx ast::Comment) -> Self::Result {
        let (start, end) = (self.ctx.start_pos.clone(), self.ctx.end_pos.clone());
        let comment_symbol = CommentOrDocSymbol::new(start, end, comment.text.clone());
        Ok(self.gs.get_symbols_mut().alloc_comment_symbol(
            comment_symbol,
            self.ctx.get_node_key(&self.ctx.cur_node),
            self.ctx.current_pkgpath.clone().unwrap(),
        ))
    }

    fn walk_missing_expr(&mut self, _missing_expr: &'ctx ast::MissingExpr) -> Self::Result {
        Ok(None)
    }
}

impl<'ctx> AdvancedResolver<'ctx> {
    #[inline]
    pub fn expr(&mut self, expr: &'ctx ast::NodeRef<ast::Expr>) -> ResolvedResult {
        if matches!(
            &expr.node,
            ast::Expr::Identifier(_)
                | ast::Expr::Config(_)
                | ast::Expr::Schema(_)
                | ast::Expr::ConfigIfEntry(_)
                | ast::Expr::Quant(_)
                | ast::Expr::Lambda(_)
                | ast::Expr::Call(_)
        ) {
            let (start, end) = expr.get_span_pos();
            self.ctx.start_pos = start;
            self.ctx.end_pos = end;
        }
        self.ctx.cur_node = expr.id.clone();

        if let Some(expr_ty) = self
            .ctx
            .node_ty_map
            .borrow()
            .get(&self.ctx.get_node_key(&expr.id))
        {
            match &expr_ty.kind {
                TypeKind::Schema(_) => {
                    let schema_symbol = self
                        .gs
                        .get_symbols()
                        .get_type_symbol(&expr_ty, self.get_current_module_info())
                        .ok_or(anyhow!("schema_symbol not found"))?;
                    self.ctx.schema_symbol_stack.push(Some(schema_symbol));
                }
                _ => {
                    self.ctx.schema_symbol_stack.push(None);
                }
            }
        }

        let expr_symbol = self.walk_expr(&expr.node);
        self.ctx.schema_symbol_stack.pop();

        match expr_symbol {
            Ok(None) => match self
                .ctx
                .node_ty_map
                .borrow()
                .get(&self.ctx.get_node_key(&expr.id))
            {
                Some(ty) => match expr.node {
                    ast::Expr::Missing(_)
                    | ast::Expr::Binary(_)
                    | ast::Expr::CompClause(_)
                    | ast::Expr::Keyword(_)
                    | ast::Expr::Arguments(_)
                    | ast::Expr::Compare(_) => return Ok(None),
                    _ => {
                        let (_, end) = expr.get_span_pos();
                        let mut expr_symbol = ExpressionSymbol::new(
                            format!("@{}", expr.node.get_expr_name()),
                            end.clone(),
                            end,
                            None,
                        );
                        expr_symbol.sema_info.ty = if matches!(&expr.node, | ast::Expr::Call(_)) {
                            if let TypeKind::Function(func_ty) = &ty.kind {
                                Some(func_ty.return_ty.clone())
                            } else {
                                Some(ty.clone())
                            }
                        } else {
                            Some(ty.clone())
                        };

                        Ok(self.gs.get_symbols_mut().alloc_expression_symbol(
                            expr_symbol,
                            self.ctx.get_node_key(&expr.id),
                            self.ctx.current_pkgpath.clone().unwrap(),
                        ))
                    }
                },
                None => Ok(None),
            },
            res => res,
        }
    }

    #[inline]
    pub fn stmt(&mut self, stmt: &'ctx ast::NodeRef<ast::Stmt>) -> ResolvedResult {
        let (start, end) = stmt.get_span_pos();
        self.ctx.start_pos = start;
        self.ctx.end_pos = end;
        self.ctx.cur_node = stmt.id.clone();
        let result = self.walk_stmt(&stmt.node);
        result
    }

    fn resolve_names(&mut self, names: &[ast::Node<String>], maybe_def: bool) -> ResolvedResult {
        let first_name = names.get(0).unwrap();
        let cur_scope = *self.ctx.scopes.last().unwrap();

        let mut first_symbol = self.gs.look_up_symbol(
            &first_name.node,
            cur_scope,
            self.get_current_module_info(),
            maybe_def,
            !self.ctx.in_config_r_value,
        );
        if first_symbol.is_none() {
            // Maybe import package symbol
            let module_info = self.get_current_module_info().unwrap();

            let import_info = module_info.get_import_info(&first_name.node);
            if import_info.is_some() {
                first_symbol = self
                    .gs
                    .get_symbols()
                    .get_symbol_by_fully_qualified_name(&import_info.unwrap().fully_qualified_name);
            }

            if let Some(first_symbol) = first_symbol {
                if self
                    .gs
                    .get_symbols()
                    .get_symbol(first_symbol)
                    .ok_or(anyhow!("first name symbol not found"))?
                    .get_sema_info()
                    .ty
                    .is_none()
                {
                    if let Some(ty) = self
                        .ctx
                        .node_ty_map
                        .borrow()
                        .get(&self.ctx.get_node_key(&first_name.id))
                    {
                        self.gs
                            .get_symbols_mut()
                            .set_symbol_type(first_symbol, ty.clone());
                    }
                }
            }
        }
        match first_symbol {
            Some(symbol_ref) => {
                let mut ret_symbol = symbol_ref.clone();
                let (start_pos, end_pos): Range = first_name.get_span_pos();
                let def_symbol = self
                    .gs
                    .get_symbols()
                    .get_symbol(symbol_ref)
                    .ok_or(anyhow!("first name symbol not found"))?;
                let (def_start_pos, def_end_pos) = def_symbol.get_range();

                let cur_scope = *self.ctx.scopes.last().unwrap();
                let ast_id = first_name.id.clone();
                let mut first_unresolved = UnresolvedSymbol::new(
                    first_name.node.clone(),
                    start_pos.clone(),
                    end_pos.clone(),
                    None,
                    self.ctx.is_type_expr,
                );
                let name = def_symbol.get_name();
                first_unresolved.def = Some(symbol_ref);
                let first_unresolved_ref = self.gs.get_symbols_mut().alloc_unresolved_symbol(
                    first_unresolved,
                    self.ctx.get_node_key(&ast_id),
                    self.ctx.current_pkgpath.clone().unwrap(),
                );

                match cur_scope.get_kind() {
                    crate::core::scope::ScopeKind::Local => {
                        let local_scope = self
                            .gs
                            .get_scopes()
                            .try_get_local_scope(&cur_scope)
                            .unwrap();
                        match local_scope.get_kind() {
                            LocalSymbolScopeKind::Config => {
                                if let crate::core::symbol::SymbolKind::Attribute =
                                    symbol_ref.get_kind()
                                {
                                    if maybe_def {
                                        self.gs.get_scopes_mut().add_def_to_scope(
                                            cur_scope,
                                            name,
                                            first_unresolved_ref,
                                        );
                                        ret_symbol = first_unresolved_ref;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }

                if def_start_pos != start_pos || def_end_pos != end_pos {
                    self.gs
                        .get_symbols_mut()
                        .set_def_and_ref(symbol_ref, first_unresolved_ref);

                    let cur_scope = *self.ctx.scopes.last().unwrap();
                    self.gs
                        .get_scopes_mut()
                        .add_ref_to_scope(cur_scope, first_unresolved_ref);
                }

                if names.len() > 1 {
                    let mut parent_ty = match self
                        .ctx
                        .node_ty_map
                        .borrow()
                        .get(&self.ctx.get_node_key(&first_name.id))
                    {
                        Some(ty) => ty.clone(),
                        None => return Ok(None),
                    };

                    for index in 1..names.len() {
                        let name = names.get(index).unwrap();
                        let def_symbol_ref = match self.gs.get_symbols().get_type_attribute(
                            &parent_ty,
                            &name.node,
                            self.get_current_module_info(),
                        ) {
                            Some(symbol) => symbol,
                            None => return Ok(None),
                        };

                        let (start_pos, end_pos): Range = name.get_span_pos();
                        let ast_id = name.id.clone();
                        let mut unresolved = UnresolvedSymbol::new(
                            name.node.clone(),
                            start_pos,
                            end_pos,
                            None,
                            self.ctx.is_type_expr,
                        );

                        unresolved.sema_info = SymbolSemanticInfo {
                            ty: self
                                .ctx
                                .node_ty_map
                                .borrow()
                                .get(&self.ctx.get_node_key(&name.id))
                                .map(|ty| ty.clone()),
                            doc: None,
                        };

                        let unresolved_ref = self.gs.get_symbols_mut().alloc_unresolved_symbol(
                            unresolved,
                            self.ctx.get_node_key(&ast_id),
                            self.ctx.current_pkgpath.clone().unwrap(),
                        );

                        self.gs
                            .get_symbols_mut()
                            .set_def_and_ref(def_symbol_ref, unresolved_ref);

                        let cur_scope = *self.ctx.scopes.last().unwrap();
                        self.gs
                            .get_scopes_mut()
                            .add_ref_to_scope(cur_scope, unresolved_ref);

                        parent_ty = match self
                            .ctx
                            .node_ty_map
                            .borrow()
                            .get(&self.ctx.get_node_key(&name.id))
                        {
                            Some(ty) => ty.clone(),
                            None => return Ok(None),
                        };
                        if index == names.len() - 1 {
                            return Ok(Some(unresolved_ref));
                        }
                    }
                }
                Ok(Some(ret_symbol))
            }
            None => {
                if maybe_def {
                    let (start_pos, end_pos): Range = first_name.get_span_pos();
                    let ast_id = first_name.id.clone();
                    let first_value = self.gs.get_symbols_mut().alloc_value_symbol(
                        ValueSymbol::new(first_name.node.clone(), start_pos, end_pos, None, false),
                        self.ctx.get_node_key(&ast_id),
                        self.ctx.current_pkgpath.clone().unwrap(),
                    );
                    self.gs.get_scopes_mut().add_def_to_scope(
                        cur_scope,
                        first_name.node.clone(),
                        first_value,
                    );

                    if let Some(symbol) = self
                        .gs
                        .get_symbols_mut()
                        .values
                        .get_mut(first_value.get_id())
                    {
                        symbol.sema_info = SymbolSemanticInfo {
                            ty: self
                                .ctx
                                .node_ty_map
                                .borrow()
                                .get(&self.ctx.get_node_key(&first_name.id))
                                .map(|ty| ty.clone()),
                            doc: None,
                        };
                    }

                    for index in 1..names.len() {
                        let name = names.get(index).unwrap();
                        let (start_pos, end_pos): Range = name.get_span_pos();
                        let ast_id = name.id.clone();
                        let value = self.gs.get_symbols_mut().alloc_value_symbol(
                            ValueSymbol::new(name.node.clone(), start_pos, end_pos, None, false),
                            self.ctx.get_node_key(&ast_id),
                            self.ctx.current_pkgpath.clone().unwrap(),
                        );

                        self.gs.get_scopes_mut().add_def_to_scope(
                            cur_scope,
                            name.node.clone(),
                            value,
                        );

                        if let Some(symbol) =
                            self.gs.get_symbols_mut().values.get_mut(value.get_id())
                        {
                            symbol.sema_info = SymbolSemanticInfo {
                                ty: self
                                    .ctx
                                    .node_ty_map
                                    .borrow()
                                    .get(&self.ctx.get_node_key(&name.id))
                                    .map(|ty| ty.clone()),
                                doc: None,
                            };
                        }
                        if index == names.len() - 1 {
                            return Ok(Some(value));
                        }
                    }
                }
                Ok(None)
            }
        }
    }

    fn resolve_target(&mut self, target: &'ctx ast::Target) -> ResolvedResult {
        let first_name = &target.name;
        let cur_scope = *self.ctx.scopes.last().unwrap();

        let first_symbol = self.gs.look_up_symbol(
            &first_name.node,
            cur_scope,
            self.get_current_module_info(),
            true,
            !self.ctx.in_config_r_value,
        );
        match first_symbol {
            Some(symbol_ref) => {
                let (start_pos, end_pos): Range = first_name.get_span_pos();
                let (def_start_pos, def_end_pos) = self
                    .gs
                    .get_symbols()
                    .get_symbol(symbol_ref)
                    .ok_or(anyhow!("first name symbol not found"))?
                    .get_range();

                // Get an unresolved symbol
                if def_start_pos != start_pos || def_end_pos != end_pos {
                    let ast_id = first_name.id.clone();
                    let mut first_unresolved = UnresolvedSymbol::new(
                        first_name.node.clone(),
                        start_pos,
                        end_pos,
                        None,
                        self.ctx.is_type_expr,
                    );
                    first_unresolved.def = Some(symbol_ref);
                    let first_unresolved_ref = self.gs.get_symbols_mut().alloc_unresolved_symbol(
                        first_unresolved,
                        self.ctx.get_node_key(&ast_id),
                        self.ctx.current_pkgpath.clone().unwrap(),
                    );
                    let cur_scope = *self.ctx.scopes.last().unwrap();
                    self.gs
                        .get_scopes_mut()
                        .add_ref_to_scope(cur_scope, first_unresolved_ref);
                }
                if !target.paths.is_empty() {
                    let mut parent_ty = match self
                        .ctx
                        .node_ty_map
                        .borrow()
                        .get(&self.ctx.get_node_key(&first_name.id))
                    {
                        Some(ty) => ty.clone(),
                        None => return Ok(None),
                    };

                    for (index, path) in target.paths.iter().enumerate() {
                        match path {
                            ast::MemberOrIndex::Member(member) => {
                                let name = member;
                                let def_symbol_ref = match self.gs.get_symbols().get_type_attribute(
                                    &parent_ty,
                                    &name.node,
                                    self.get_current_module_info(),
                                ) {
                                    Some(symbol) => symbol,
                                    None => return Ok(None),
                                };

                                let (start_pos, end_pos): Range = name.get_span_pos();
                                let ast_id = name.id.clone();
                                let mut unresolved = UnresolvedSymbol::new(
                                    name.node.clone(),
                                    start_pos,
                                    end_pos,
                                    None,
                                    self.ctx.is_type_expr,
                                );
                                unresolved.def = Some(def_symbol_ref);
                                unresolved.sema_info = SymbolSemanticInfo {
                                    ty: self
                                        .ctx
                                        .node_ty_map
                                        .borrow()
                                        .get(&self.ctx.get_node_key(&name.id))
                                        .map(|ty| ty.clone()),
                                    doc: None,
                                };

                                let unresolved_ref =
                                    self.gs.get_symbols_mut().alloc_unresolved_symbol(
                                        unresolved,
                                        self.ctx.get_node_key(&ast_id),
                                        self.ctx.current_pkgpath.clone().unwrap(),
                                    );

                                let cur_scope = *self.ctx.scopes.last().unwrap();
                                self.gs
                                    .get_scopes_mut()
                                    .add_ref_to_scope(cur_scope, unresolved_ref);

                                parent_ty = match self
                                    .ctx
                                    .node_ty_map
                                    .borrow()
                                    .get(&self.ctx.get_node_key(&name.id))
                                {
                                    Some(ty) => ty.clone(),
                                    None => return Ok(None),
                                };
                                if index == target.paths.len() - 1 {
                                    return Ok(Some(unresolved_ref));
                                }
                            }
                            ast::MemberOrIndex::Index(index_expr) => {
                                let last_maybe_def = self.ctx.maybe_def;
                                self.ctx.maybe_def = false;
                                let symbol = self.expr(index_expr);
                                parent_ty = match self
                                    .ctx
                                    .node_ty_map
                                    .borrow()
                                    .get(&self.ctx.get_node_key(&index_expr.id))
                                {
                                    Some(ty) => ty.clone(),
                                    None => return Ok(None),
                                };
                                self.ctx.maybe_def = last_maybe_def;
                                if index == target.paths.len() {
                                    return symbol;
                                }
                            }
                        }
                    }
                }
                Ok(Some(symbol_ref))
            }
            None => {
                let (start_pos, end_pos): Range = first_name.get_span_pos();
                let ast_id = first_name.id.clone();
                let first_value = self.gs.get_symbols_mut().alloc_value_symbol(
                    ValueSymbol::new(first_name.node.clone(), start_pos, end_pos, None, false),
                    self.ctx.get_node_key(&ast_id),
                    self.ctx.current_pkgpath.clone().unwrap(),
                );
                self.gs.get_scopes_mut().add_def_to_scope(
                    cur_scope,
                    first_name.node.clone(),
                    first_value,
                );

                if let Some(symbol) = self
                    .gs
                    .get_symbols_mut()
                    .values
                    .get_mut(first_value.get_id())
                {
                    symbol.sema_info = SymbolSemanticInfo {
                        ty: self
                            .ctx
                            .node_ty_map
                            .borrow()
                            .get(&self.ctx.get_node_key(&first_name.id))
                            .map(|ty| ty.clone()),
                        doc: None,
                    };
                }

                for (index, path) in target.paths.iter().enumerate() {
                    match path {
                        ast::MemberOrIndex::Member(member) => {
                            let name = member;
                            let (start_pos, end_pos): Range = name.get_span_pos();
                            let ast_id = name.id.clone();
                            let value = self.gs.get_symbols_mut().alloc_value_symbol(
                                ValueSymbol::new(
                                    name.node.clone(),
                                    start_pos,
                                    end_pos,
                                    None,
                                    false,
                                ),
                                self.ctx.get_node_key(&ast_id),
                                self.ctx.current_pkgpath.clone().unwrap(),
                            );

                            self.gs.get_scopes_mut().add_def_to_scope(
                                cur_scope,
                                name.node.clone(),
                                value,
                            );

                            if let Some(symbol) =
                                self.gs.get_symbols_mut().values.get_mut(value.get_id())
                            {
                                symbol.sema_info = SymbolSemanticInfo {
                                    ty: self
                                        .ctx
                                        .node_ty_map
                                        .borrow()
                                        .get(&self.ctx.get_node_key(&name.id))
                                        .map(|ty| ty.clone()),
                                    doc: None,
                                };
                            }
                            if index == target.paths.len() {
                                return Ok(Some(value));
                            }
                        }
                        ast::MemberOrIndex::Index(index_expr) => {
                            let last_maybe_def = self.ctx.maybe_def;
                            self.ctx.maybe_def = false;
                            let symbol = self.expr(index_expr);
                            self.ctx.maybe_def = last_maybe_def;
                            if index == target.paths.len() {
                                return symbol;
                            }
                        }
                    }
                }
                Ok(None)
            }
        }
    }

    #[inline]
    pub fn walk_target_expr(&mut self, target: &'ctx ast::NodeRef<ast::Target>) -> ResolvedResult {
        self.walk_target_expr_with_hint(target, false)
    }

    pub fn walk_target_expr_with_hint(
        &mut self,
        target: &'ctx ast::NodeRef<ast::Target>,
        with_hint: bool,
    ) -> ResolvedResult {
        let symbol_ref = if let Some(identifier_symbol) = self
            .gs
            .get_symbols()
            .symbols_info
            .node_symbol_map
            .get(&self.ctx.get_node_key(&&target.id))
            .map(|symbol_ref| *symbol_ref)
        {
            if let Some(symbol) = self
                .gs
                .get_symbols_mut()
                .values
                .get_mut(identifier_symbol.get_id())
            {
                let id = if let Some(last) = target.node.paths.last() {
                    last.id()
                } else {
                    target.node.name.id.clone()
                };
                let ty = self
                    .ctx
                    .node_ty_map
                    .borrow()
                    .get(&self.ctx.get_node_key(&id))
                    .map(|ty| ty.clone());
                if with_hint {
                    symbol.hint = ty.as_ref().map(|ty| SymbolHint {
                        kind: SymbolHintKind::TypeHint(ty.ty_hint()),
                        pos: symbol.get_range().1,
                    });
                }
                symbol.sema_info = SymbolSemanticInfo { ty, doc: None };
            }

            let cur_scope = *self.ctx.scopes.last().unwrap();
            match cur_scope.kind {
                crate::core::scope::ScopeKind::Local => {
                    self.gs.get_scopes_mut().add_def_to_scope(
                        cur_scope,
                        target.node.get_name().to_string(),
                        identifier_symbol,
                    );
                }
                crate::core::scope::ScopeKind::Root => {}
            }
            identifier_symbol
        } else {
            match self.resolve_target(&target.node)? {
                Some(symbol) => symbol,
                None => return Ok(None),
            }
        };

        Ok(Some(symbol_ref))
    }

    #[inline]
    pub fn walk_identifier_expr(
        &mut self,
        identifier: &'ctx ast::NodeRef<ast::Identifier>,
    ) -> ResolvedResult {
        self.walk_identifier_expr_with_hint(identifier, false)
    }

    pub fn walk_identifier_expr_with_hint(
        &mut self,
        identifier: &'ctx ast::NodeRef<ast::Identifier>,
        with_hint: bool,
    ) -> ResolvedResult {
        let symbol_ref = if let Some(identifier_symbol) = self
            .gs
            .get_symbols()
            .symbols_info
            .node_symbol_map
            .get(&self.ctx.get_node_key(&&identifier.id))
            .map(|symbol_ref| *symbol_ref)
        {
            if let Some(symbol) = self
                .gs
                .get_symbols_mut()
                .values
                .get_mut(identifier_symbol.get_id())
            {
                let id = if identifier.node.names.is_empty() {
                    &identifier.id
                } else {
                    &identifier.node.names.last().unwrap().id
                };
                let ty = self
                    .ctx
                    .node_ty_map
                    .borrow()
                    .get(&self.ctx.get_node_key(id))
                    .map(|ty| ty.clone());
                if with_hint {
                    symbol.hint = ty.as_ref().map(|ty| SymbolHint {
                        kind: SymbolHintKind::TypeHint(ty.ty_hint()),
                        pos: symbol.get_range().1,
                    });
                }
                symbol.sema_info = SymbolSemanticInfo { ty, doc: None };
            }

            if self.ctx.maybe_def && identifier.node.names.len() > 0 {
                let cur_scope = *self.ctx.scopes.last().unwrap();
                match cur_scope.kind {
                    crate::core::scope::ScopeKind::Local => {
                        self.gs.get_scopes_mut().add_def_to_scope(
                            cur_scope,
                            identifier.node.names.last().unwrap().node.clone(),
                            identifier_symbol,
                        );
                    }
                    crate::core::scope::ScopeKind::Root => {}
                }
            }
            identifier_symbol
        } else {
            match self.resolve_names(&identifier.node.names, self.ctx.maybe_def)? {
                Some(symbol) => symbol,
                None => return Ok(None),
            }
        };

        Ok(Some(symbol_ref))
    }

    pub fn walk_type_expr(
        &mut self,
        ty_node: Option<&'ctx ast::Node<ast::Type>>,
    ) -> ResolvedResult {
        self.ctx.is_type_expr = true;
        if let Some(ty_node) = ty_node {
            match &ty_node.node {
                ast::Type::Any => {}
                ast::Type::Named(identifier) => {
                    self.walk_identifier(identifier)?;
                }
                ast::Type::Basic(_) => {}
                ast::Type::List(list_type) => {
                    self.walk_type_expr(list_type.inner_type.as_ref().map(|ty| ty.as_ref()))?;
                }
                ast::Type::Dict(dict_type) => {
                    self.walk_type_expr(dict_type.key_type.as_ref().map(|ty| ty.as_ref()))?;
                    self.walk_type_expr(dict_type.value_type.as_ref().map(|ty| ty.as_ref()))?;
                }
                ast::Type::Union(union_type) => {
                    for elem_ty in union_type.type_elements.iter() {
                        self.walk_type_expr(Some(elem_ty))?;
                    }
                }
                ast::Type::Literal(_) => {}
                ast::Type::Function(func_type) => {
                    if let Some(params_ty) = &func_type.params_ty {
                        for param_ty in params_ty.iter() {
                            self.walk_type_expr(Some(param_ty))?;
                        }
                    }
                    if let Some(ret_ty) = &func_type.ret_ty {
                        self.walk_type_expr(Some(&ret_ty))?;
                    }
                }
            }
        }

        if let Some(ty_node) = ty_node {
            match self
                .ctx
                .node_ty_map
                .borrow()
                .get(&self.ctx.get_node_key(&ty_node.id))
            {
                Some(ty) => {
                    let (_, end) = ty_node.get_span_pos();
                    let mut expr_symbol =
                        ExpressionSymbol::new(format!("@{}", ty.ty_hint()), end.clone(), end, None);

                    expr_symbol.sema_info.ty = Some(ty.clone());
                    self.gs.get_symbols_mut().alloc_expression_symbol(
                        expr_symbol,
                        self.ctx.get_node_key(&ty_node.id),
                        self.ctx.current_pkgpath.clone().unwrap(),
                    );
                }
                None => {}
            }
        }
        self.ctx.is_type_expr = false;
        Ok(None)
    }

    pub fn do_arguments_symbol_resolve(
        &mut self,
        args: &'ctx [ast::NodeRef<ast::Expr>],
        kwargs: &'ctx [ast::NodeRef<ast::Keyword>],
    ) -> anyhow::Result<()> {
        for arg in args.iter() {
            self.expr(arg)?;
        }
        for kw in kwargs.iter() {
            if let Some(value) = &kw.node.value {
                self.expr(value)?;
            }
            let (start_pos, end_pos): Range = kw.node.arg.get_span_pos();
            let value = self.gs.get_symbols_mut().alloc_value_symbol(
                ValueSymbol::new(kw.node.arg.node.get_name(), start_pos, end_pos, None, false),
                self.ctx.get_node_key(&kw.id),
                self.ctx.current_pkgpath.clone().unwrap(),
            );

            if let Some(value) = self.gs.get_symbols_mut().values.get_mut(value.get_id()) {
                value.sema_info = SymbolSemanticInfo {
                    ty: self
                        .ctx
                        .node_ty_map
                        .borrow()
                        .get(&self.ctx.get_node_key(&kw.id))
                        .map(|ty| ty.clone()),
                    doc: None,
                };
            }
        }
        Ok(())
    }

    pub fn do_arguments_symbol_resolve_with_hint(
        &mut self,
        args: &'ctx [ast::NodeRef<ast::Expr>],
        kwargs: &'ctx [ast::NodeRef<ast::Keyword>],
        params: &[Parameter],
        with_hint: bool,
    ) -> anyhow::Result<()> {
        if params.is_empty() {
            self.do_arguments_symbol_resolve(args, kwargs)?;
        } else {
            for (arg, param) in args.iter().zip(params.iter()) {
                self.expr(arg)?;

                if with_hint {
                    let symbol_data = self.gs.get_symbols_mut();
                    let id = match &arg.node {
                        ast::Expr::Identifier(id) => id.names.last().unwrap().id.clone(),
                        _ => arg.id.clone(),
                    };
                    if let Some(arg_ref) = symbol_data
                        .symbols_info
                        .node_symbol_map
                        .get(&self.ctx.get_node_key(&id))
                    {
                        match arg_ref.get_kind() {
                            crate::core::symbol::SymbolKind::Expression => {
                                if let Some(expr) = symbol_data.exprs.get_mut(arg_ref.get_id()) {
                                    expr.hint = Some(SymbolHint {
                                        pos: arg.get_pos(),
                                        kind: SymbolHintKind::VarHint(param.name.clone()),
                                    });
                                }
                            }
                            crate::core::symbol::SymbolKind::Unresolved => {
                                let mut has_hint = false;
                                if let Some(unresolved) =
                                    symbol_data.unresolved.get(arg_ref.get_id())
                                {
                                    if let Some(def) = unresolved.def {
                                        if let Some(def) = symbol_data.get_symbol(def) {
                                            if def.get_name() != param.name {
                                                has_hint = true;
                                            }
                                        }
                                    }
                                }
                                if has_hint {
                                    if let Some(unresolved) =
                                        symbol_data.unresolved.get_mut(arg_ref.get_id())
                                    {
                                        unresolved.hint = Some(SymbolHint {
                                            kind: SymbolHintKind::VarHint(param.name.clone()),
                                            pos: arg.get_pos(),
                                        });
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            for kw in kwargs.iter() {
                if let Some(value) = &kw.node.value {
                    self.expr(value)?;
                }
                let (start_pos, end_pos): Range = kw.node.arg.get_span_pos();
                let value = self.gs.get_symbols_mut().alloc_value_symbol(
                    ValueSymbol::new(kw.node.arg.node.get_name(), start_pos, end_pos, None, false),
                    self.ctx.get_node_key(&kw.id),
                    self.ctx.current_pkgpath.clone().unwrap(),
                );

                if let Some(value) = self.gs.get_symbols_mut().values.get_mut(value.get_id()) {
                    value.sema_info = SymbolSemanticInfo {
                        ty: self
                            .ctx
                            .node_ty_map
                            .borrow()
                            .get(&self.ctx.get_node_key(&kw.id))
                            .map(|ty| ty.clone()),
                        doc: None,
                    };
                }
            }
        }
        Ok(())
    }

    pub(crate) fn walk_config_entries(
        &mut self,
        entries: &'ctx [ast::NodeRef<ast::ConfigEntry>],
    ) -> anyhow::Result<()> {
        let (start, end) = (self.ctx.start_pos.clone(), self.ctx.end_pos.clone());

        let schema_symbol = self.ctx.schema_symbol_stack.last().unwrap_or(&None).clone();
        let kind = LocalSymbolScopeKind::Config;

        self.enter_local_scope(
            &self.ctx.current_filename.as_ref().unwrap().clone(),
            start,
            end,
            kind,
        );

        if let Some(owner) = schema_symbol {
            let cur_scope = self.ctx.scopes.last().unwrap();
            self.gs
                .get_scopes_mut()
                .set_owner_to_scope(*cur_scope, owner);
        }

        let mut entries_range = vec![];
        for entry in entries.iter() {
            entries_range.push((
                entry.node.key.clone().map(|k| k.get_span_pos()),
                entry.node.value.get_span_pos(),
            ));
            self.ctx.in_config_r_value = true;
            self.expr(&entry.node.value)?;
            self.ctx.in_config_r_value = false;

            if let Some(key) = &entry.node.key {
                self.ctx.maybe_def = true;
                if let Some(symbol_ref) = self.expr(key)? {
                    let config_key_symbol =
                        self.gs.get_symbols().unresolved.get(symbol_ref.get_id());
                    let hint: Option<SymbolHint> =
                        if let Some(config_key_symbol) = config_key_symbol {
                            match config_key_symbol.get_definition() {
                                Some(def_ref) => match self.gs.get_symbols().get_symbol(def_ref) {
                                    Some(def_symbol) => {
                                        let ty = def_symbol.get_sema_info().ty.clone();
                                        ty.as_ref().map(|ty| SymbolHint {
                                            kind: SymbolHintKind::TypeHint(ty.ty_hint()),
                                            pos: config_key_symbol.get_range().1.clone(),
                                        })
                                    }
                                    None => None,
                                },
                                None => None,
                            }
                        } else {
                            None
                        };
                    if let Some(config_key_symbol_mut_ref) = self
                        .gs
                        .get_symbols_mut()
                        .unresolved
                        .get_mut(symbol_ref.get_id())
                    {
                        config_key_symbol_mut_ref.hint = hint;
                    }
                }
                self.ctx.maybe_def = false;
            }
        }

        let cur_scope = self.ctx.scopes.last().unwrap();
        self.gs
            .get_scopes_mut()
            .config_scope_context
            .insert(cur_scope.get_id(), ConfigScopeContext { entries_range });
        self.leave_scope();
        Ok(())
    }

    pub(crate) fn resolve_decorator(&mut self, decorators: &'ctx [ast::NodeRef<ast::CallExpr>]) {
        for decorator in decorators {
            let func_ident = &decorator.node.func;
            let (start, end) = func_ident.get_span_pos();
            if let kclvm_ast::ast::Expr::Identifier(id) = &func_ident.node {
                let decorator_symbol = DecoratorSymbol::new(start, end, id.get_name());
                self.gs.get_symbols_mut().alloc_decorator_symbol(
                    decorator_symbol,
                    self.ctx.get_node_key(&self.ctx.cur_node),
                    self.ctx.current_pkgpath.clone().unwrap(),
                );
            }
        }
    }
}
