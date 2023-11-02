use kclvm_ast::ast;
use kclvm_ast::pos::GetPos;
use kclvm_ast::walker::MutSelfTypedResultWalker;
use kclvm_error::diagnostic::Range;

use crate::core::symbol::{SymbolRef, UnresolvedSymbol, ValueSymbol};

use super::AdvancedResolver;

type ResolvedResult = Option<SymbolRef>;

impl<'ctx> MutSelfTypedResultWalker<'ctx> for AdvancedResolver<'ctx> {
    type Result = Option<SymbolRef>;

    fn walk_module(&mut self, module: &'ctx ast::Module) -> Self::Result {
        for stmt in module.body.iter() {
            self.stmt(&stmt);
        }
        None
    }

    fn walk_expr_stmt(&mut self, expr_stmt: &'ctx ast::ExprStmt) -> Self::Result {
        for expr in expr_stmt.exprs.iter() {
            self.expr(&expr);
        }
        None
    }

    fn walk_unification_stmt(
        &mut self,
        unification_stmt: &'ctx ast::UnificationStmt,
    ) -> Self::Result {
        self.ctx.maybe_def = true;
        self.walk_identifier_expr(&unification_stmt.target);
        self.ctx.maybe_def = false;
        self.walk_schema_expr(&unification_stmt.value.node);
        None
    }

    fn walk_type_alias_stmt(&mut self, type_alias_stmt: &'ctx ast::TypeAliasStmt) -> Self::Result {
        let alias_symbol = self.gs.get_symbols().get_symbol_by_fully_qualified_name(
            &(self.ctx.current_pkgpath.as_ref().unwrap().clone()
                + "."
                + &type_alias_stmt.type_name.node.get_name()),
        )?;
        self.update_symbol_info_by_node(alias_symbol, &type_alias_stmt.type_name);
        self.walk_type_expr(Some(&type_alias_stmt.ty));
        None
    }

    fn walk_assign_stmt(&mut self, assign_stmt: &'ctx ast::AssignStmt) -> Self::Result {
        for target in &assign_stmt.targets {
            if target.node.names.is_empty() {
                continue;
            }
            self.ctx.maybe_def = true;
            self.walk_identifier_expr(target);
            self.ctx.maybe_def = false;
        }
        self.walk_type_expr(assign_stmt.ty.as_ref().map(|ty| ty.as_ref()));
        self.expr(&assign_stmt.value);
        None
    }

    fn walk_aug_assign_stmt(&mut self, aug_assign_stmt: &'ctx ast::AugAssignStmt) -> Self::Result {
        self.ctx.maybe_def = true;
        self.walk_identifier_expr(&aug_assign_stmt.target);
        self.ctx.maybe_def = false;
        self.expr(&aug_assign_stmt.value);
        None
    }

    fn walk_assert_stmt(&mut self, assert_stmt: &'ctx ast::AssertStmt) -> Self::Result {
        self.expr(&assert_stmt.test);
        if let Some(if_cond) = &assert_stmt.if_cond {
            self.expr(if_cond);
        }
        None
    }

    fn walk_if_stmt(&mut self, if_stmt: &'ctx ast::IfStmt) -> Self::Result {
        self.expr(&if_stmt.cond);
        for stmt in if_stmt.body.iter() {
            self.stmt(stmt);
        }
        for stmt in if_stmt.orelse.iter() {
            self.stmt(stmt);
        }
        None
    }

    fn walk_import_stmt(&mut self, import_stmt: &'ctx ast::ImportStmt) -> Self::Result {
        let ast_id = self.ctx.cur_node.clone();
        let (start_pos, end_pos) = (self.ctx.start_pos.clone(), self.ctx.end_pos.clone());
        let mut unresolved =
            UnresolvedSymbol::new(import_stmt.path.clone(), start_pos, end_pos, None);
        let package_symbol = self
            .gs
            .get_symbols()
            .get_symbol_by_fully_qualified_name(&import_stmt.path)?;
        unresolved.def = Some(package_symbol);
        let unresolved_ref = self
            .gs
            .get_symbols_mut()
            .alloc_unresolved_symbol(unresolved, &ast_id);
        self.gs
            .get_symbols_mut()
            .symbols_info
            .ast_id_map
            .insert(ast_id, unresolved_ref);
        let cur_scope = *self.ctx.scopes.last().unwrap();
        self.gs
            .get_scopes_mut()
            .add_ref_to_scope(cur_scope, unresolved_ref);
        Some(unresolved_ref)
    }

    fn walk_schema_stmt(&mut self, schema_stmt: &'ctx ast::SchemaStmt) -> Self::Result {
        let schema_ty = self.ctx.node_ty_map.get(&schema_stmt.name.id)?.clone();
        let schema_symbol = self
            .gs
            .get_symbols()
            .get_type_symbol(&schema_ty, self.get_current_module_info())?;

        self.update_symbol_info_by_node(schema_symbol, &schema_stmt.name);
        let (start, end) = schema_stmt.get_span_pos();
        self.enter_local_scope(&self.ctx.current_filename.clone().unwrap(), start, end);
        let cur_scope = *self.ctx.scopes.last().unwrap();
        self.gs
            .get_scopes_mut()
            .set_owner_to_scope(cur_scope, schema_symbol);
        if let Some(parent) = &schema_stmt.parent_name {
            self.gs
                .get_symbols_mut()
                .schemas
                .get_mut(schema_symbol.get_id())?
                .parent_schema = self.walk_identifier_expr(parent);
        }
        if let Some(for_host) = &schema_stmt.for_host_name {
            self.gs
                .get_symbols_mut()
                .schemas
                .get_mut(schema_symbol.get_id())?
                .for_host = self.walk_identifier_expr(for_host);
        }
        let mut mixins = vec![];
        for mixin in schema_stmt.mixins.iter() {
            mixins.push(self.walk_identifier_expr(mixin)?);
        }
        self.gs
            .get_symbols_mut()
            .schemas
            .get_mut(schema_symbol.get_id())?
            .mixins = mixins;

        if let Some(args) = &schema_stmt.args {
            self.walk_arguments(&args.node);
        }
        if let Some(index_signature) = &schema_stmt.index_signature {
            if let Some(key_name) = &index_signature.node.key_name {
                let (start, end) = index_signature.get_span_pos();
                let value = self.gs.get_symbols_mut().alloc_value_symbol(
                    ValueSymbol::new(key_name.clone(), start, end, Some(schema_symbol), false),
                    &index_signature.id,
                );
                self.update_symbol_info_by_node(value, index_signature);
                self.gs
                    .get_scopes_mut()
                    .add_def_to_scope(cur_scope, key_name.clone(), value);

                self.walk_type_expr(Some(&index_signature.node.value_ty));
                if let Some(value) = &index_signature.node.value {
                    self.expr(value);
                };
            }
        }
        for stmt in schema_stmt.body.iter() {
            if let Some(attribute_symbol) = self.stmt(&stmt) {
                let name = self
                    .gs
                    .get_symbols()
                    .get_symbol(attribute_symbol)?
                    .get_name();
                self.gs
                    .get_symbols_mut()
                    .schemas
                    .get_mut(schema_symbol.get_id())?
                    .attributes
                    .insert(name, attribute_symbol);
            }
        }

        for check_expr in schema_stmt.checks.iter() {
            self.walk_check_expr(&check_expr.node);
        }
        self.leave_scope();

        Some(schema_symbol)
    }

    fn walk_rule_stmt(&mut self, rule_stmt: &'ctx ast::RuleStmt) -> Self::Result {
        let rule_ty = self.ctx.node_ty_map.get(&rule_stmt.name.id)?.clone();
        let rule_symbol = self
            .gs
            .get_symbols()
            .get_type_symbol(&rule_ty, self.get_current_module_info())?;
        self.update_symbol_info_by_node(rule_symbol, &rule_stmt.name);

        if let Some(for_host) = &rule_stmt.for_host_name {
            self.gs
                .get_symbols_mut()
                .rules
                .get_mut(rule_symbol.get_id())?
                .for_host = self.walk_identifier_expr(for_host);
        }
        let mut parent_rules = vec![];
        for parent_rule in rule_stmt.parent_rules.iter() {
            parent_rules.push(self.walk_identifier_expr(parent_rule)?);
        }
        self.gs
            .get_symbols_mut()
            .rules
            .get_mut(rule_symbol.get_id())?
            .parent_rules = parent_rules;
        Some(rule_symbol)
    }

    fn walk_quant_expr(&mut self, quant_expr: &'ctx ast::QuantExpr) -> Self::Result {
        self.expr(&quant_expr.target);
        let (start, mut end) = quant_expr.test.get_span_pos();
        if let Some(if_cond) = &quant_expr.if_cond {
            end = if_cond.get_end_pos();
        }
        self.enter_local_scope(
            &self.ctx.current_filename.as_ref().unwrap().clone(),
            start,
            end,
        );
        let cur_scope = *self.ctx.scopes.last().unwrap();
        for target in quant_expr.variables.iter() {
            if target.node.names.is_empty() {
                continue;
            }
            let name = target.node.get_name();
            let (start_pos, end_pos): Range = target.get_span_pos();
            let ast_id = target.id.clone();
            let value = self.gs.get_symbols_mut().alloc_value_symbol(
                ValueSymbol::new(name.clone(), start_pos, end_pos, None, false),
                &ast_id,
            );
            self.gs
                .get_scopes_mut()
                .add_def_to_scope(cur_scope, name, value);
            self.update_symbol_info_by_node(value, &target);
        }

        if let Some(if_cond) = &quant_expr.if_cond {
            self.expr(&if_cond);
        }
        self.expr(&quant_expr.test);
        self.leave_scope();
        None
    }

    fn walk_schema_attr(&mut self, schema_attr: &'ctx ast::SchemaAttr) -> Self::Result {
        let attr_symbol = *self
            .gs
            .get_symbols()
            .symbols_info
            .ast_id_map
            .get(&schema_attr.name.id)?;
        self.update_symbol_info_by_node(attr_symbol, &schema_attr.name);
        self.walk_type_expr(Some(&schema_attr.ty));
        if let Some(value) = &schema_attr.value {
            self.expr(value);
        }
        Some(attr_symbol)
    }

    /// <body> if <cond> else <orelse> -> sup([body, orelse])
    fn walk_if_expr(&mut self, if_expr: &'ctx ast::IfExpr) -> Self::Result {
        self.expr(&if_expr.cond);
        self.expr(&if_expr.body);
        self.expr(&if_expr.orelse);
        None
    }

    fn walk_unary_expr(&mut self, unary_expr: &'ctx ast::UnaryExpr) -> Self::Result {
        self.expr(&unary_expr.operand);
        None
    }

    fn walk_binary_expr(&mut self, binary_expr: &'ctx ast::BinaryExpr) -> Self::Result {
        self.expr(&binary_expr.left);
        self.expr(&binary_expr.right);
        None
    }

    fn walk_selector_expr(&mut self, selector_expr: &'ctx ast::SelectorExpr) -> Self::Result {
        self.expr(&selector_expr.value);
        let mut parent_ty = self.ctx.node_ty_map.get(&selector_expr.value.id)?.clone();
        for name in &selector_expr.attr.node.names {
            let def_symbol_ref = self.gs.get_symbols().get_type_attribute(
                &parent_ty,
                &name.node,
                self.get_current_module_info(),
            )?;

            let (start_pos, end_pos): Range = name.get_span_pos();
            let ast_id = name.id.clone();
            let mut unresolved = UnresolvedSymbol::new(name.node.clone(), start_pos, end_pos, None);
            unresolved.def = Some(def_symbol_ref);
            let unresolved_ref = self
                .gs
                .get_symbols_mut()
                .alloc_unresolved_symbol(unresolved, &ast_id);
            let cur_scope = *self.ctx.scopes.last().unwrap();
            self.gs
                .get_scopes_mut()
                .add_ref_to_scope(cur_scope, unresolved_ref);

            self.update_symbol_info_by_node(unresolved_ref, &name);
            parent_ty = self.ctx.node_ty_map.get(&name.id)?.clone();
        }
        None
    }

    fn walk_call_expr(&mut self, call_expr: &'ctx ast::CallExpr) -> Self::Result {
        self.expr(&call_expr.func);
        self.do_arguments_symbol_resolve(&call_expr.args, &call_expr.keywords);
        None
    }

    fn walk_subscript(&mut self, subscript: &'ctx ast::Subscript) -> Self::Result {
        self.expr(&subscript.value);
        if let Some(index) = &subscript.index {
            self.expr(index);
        } else {
            for expr in [&subscript.lower, &subscript.upper, &subscript.step]
                .iter()
                .copied()
                .flatten()
            {
                self.expr(expr);
            }
        }
        None
    }

    fn walk_paren_expr(&mut self, paren_expr: &'ctx ast::ParenExpr) -> Self::Result {
        self.expr(&paren_expr.expr);
        None
    }

    fn walk_list_expr(&mut self, list_expr: &'ctx ast::ListExpr) -> Self::Result {
        for expr in list_expr.elts.iter() {
            self.expr(expr);
        }
        None
    }

    fn walk_list_comp(&mut self, list_comp: &'ctx ast::ListComp) -> Self::Result {
        let start = list_comp.elt.get_pos();
        let end = match list_comp.generators.last() {
            Some(last) => last.get_end_pos(),
            None => list_comp.elt.get_end_pos(),
        };
        self.enter_local_scope(&self.ctx.current_filename.clone().unwrap(), start, end);
        for comp_clause in &list_comp.generators {
            self.walk_comp_clause(&comp_clause.node);
        }
        self.expr(&list_comp.elt);
        self.leave_scope();
        None
    }

    fn walk_dict_comp(&mut self, dict_comp: &'ctx ast::DictComp) -> Self::Result {
        let key = dict_comp.entry.key.as_ref().unwrap();
        let start = key.get_pos();
        let end = match dict_comp.generators.last() {
            Some(last) => last.get_end_pos(),
            None => dict_comp.entry.value.get_end_pos(),
        };
        self.enter_local_scope(&self.ctx.current_filename.clone().unwrap(), start, end);
        for comp_clause in &dict_comp.generators {
            self.walk_comp_clause(&comp_clause.node);
        }
        self.expr(key);
        self.expr(&dict_comp.entry.value);
        self.leave_scope();
        None
    }

    fn walk_list_if_item_expr(
        &mut self,
        list_if_item_expr: &'ctx ast::ListIfItemExpr,
    ) -> Self::Result {
        if let Some(orelse) = &list_if_item_expr.orelse {
            self.expr(orelse);
        }
        for expr in list_if_item_expr.exprs.iter() {
            self.expr(expr);
        }
        None
    }

    fn walk_starred_expr(&mut self, starred_expr: &'ctx ast::StarredExpr) -> Self::Result {
        self.expr(&starred_expr.value);
        None
    }

    fn walk_config_if_entry_expr(
        &mut self,
        config_if_entry_expr: &'ctx ast::ConfigIfEntryExpr,
    ) -> Self::Result {
        self.expr(&config_if_entry_expr.if_cond);
        self.walk_config_entries(&config_if_entry_expr.items);
        if let Some(expr) = config_if_entry_expr.orelse.as_ref() {
            self.expr(expr);
        }
        None
    }

    fn walk_comp_clause(&mut self, comp_clause: &'ctx ast::CompClause) -> Self::Result {
        self.expr(&comp_clause.iter);
        for target in comp_clause.targets.iter() {
            self.ctx.maybe_def = true;
            self.walk_identifier_expr(target);
            self.ctx.maybe_def = false;
        }
        for if_expr in comp_clause.ifs.iter() {
            self.expr(if_expr);
        }
        None
    }

    fn walk_schema_expr(&mut self, schema_expr: &'ctx ast::SchemaExpr) -> Self::Result {
        self.walk_identifier_expr(&schema_expr.name)?;
        let schema_ty = self.ctx.node_ty_map.get(&schema_expr.name.id)?.clone();
        let schema_symbol = self
            .gs
            .get_symbols()
            .get_type_symbol(&schema_ty, self.get_current_module_info())?;
        self.ctx.current_schema_symbol = Some(schema_symbol);
        self.expr(&schema_expr.config);
        self.do_arguments_symbol_resolve(&schema_expr.args, &schema_expr.kwargs);
        None
    }

    fn walk_config_expr(&mut self, config_expr: &'ctx ast::ConfigExpr) -> Self::Result {
        self.walk_config_entries(&config_expr.items);
        None
    }

    fn walk_check_expr(&mut self, check_expr: &'ctx ast::CheckExpr) -> Self::Result {
        if let Some(msg) = &check_expr.msg {
            self.expr(msg);
        }
        if let Some(if_cond) = &check_expr.if_cond {
            self.expr(if_cond);
        }
        self.expr(&check_expr.test);
        None
    }

    fn walk_lambda_expr(&mut self, lambda_expr: &'ctx ast::LambdaExpr) -> Self::Result {
        let (start, end) = (self.ctx.start_pos.clone(), self.ctx.end_pos.clone());
        self.enter_local_scope(&self.ctx.current_filename.clone().unwrap(), start, end);
        if let Some(args) = &lambda_expr.args {
            self.walk_arguments(&args.node);
        }
        if let Some(ret_annotation_ty) = &lambda_expr.return_ty {
            self.walk_type_expr(Some(&ret_annotation_ty));
        }
        for stmt in lambda_expr.body.iter() {
            self.stmt(&stmt);
        }
        self.leave_scope();
        None
    }

    fn walk_keyword(&mut self, keyword: &'ctx ast::Keyword) -> Self::Result {
        self.ctx.maybe_def = true;
        self.walk_identifier_expr(&keyword.arg);
        self.ctx.maybe_def = false;
        if let Some(value) = &keyword.value {
            self.expr(&value);
        }
        None
    }

    fn walk_arguments(&mut self, arguments: &'ctx ast::Arguments) -> Self::Result {
        for (i, arg) in arguments.args.iter().enumerate() {
            let ty = arguments.get_arg_type_node(i);
            self.walk_type_expr(ty);
            self.ctx.maybe_def = true;
            self.walk_identifier_expr(arg);
            self.ctx.maybe_def = false;

            if let Some(val) = &arguments.defaults[i] {
                self.expr(val);
            }
        }
        None
    }

    fn walk_compare(&mut self, compare: &'ctx ast::Compare) -> Self::Result {
        self.expr(&compare.left);
        for comparator in compare.comparators.iter() {
            self.expr(&comparator);
        }
        None
    }

    fn walk_identifier(&mut self, identifier: &'ctx ast::Identifier) -> Self::Result {
        let symbol_ref = self.resolve_names(&identifier.names, self.ctx.maybe_def)?;
        Some(symbol_ref)
    }

    fn walk_number_lit(&mut self, _number_lit: &'ctx ast::NumberLit) -> Self::Result {
        None
    }

    fn walk_string_lit(&mut self, _string_lit: &'ctx ast::StringLit) -> Self::Result {
        None
    }

    fn walk_name_constant_lit(
        &mut self,
        _name_constant_lit: &'ctx ast::NameConstantLit,
    ) -> Self::Result {
        None
    }

    fn walk_joined_string(&mut self, joined_string: &'ctx ast::JoinedString) -> Self::Result {
        self.ctx.maybe_def = false;
        for expr in joined_string.values.iter() {
            self.expr(expr);
        }
        None
    }

    fn walk_formatted_value(&mut self, formatted_value: &'ctx ast::FormattedValue) -> Self::Result {
        self.expr(&formatted_value.value);
        None
    }

    fn walk_comment(&mut self, _comment: &'ctx ast::Comment) -> Self::Result {
        None
    }

    fn walk_missing_expr(&mut self, _missing_expr: &'ctx ast::MissingExpr) -> Self::Result {
        None
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
        ) {
            let (start, end) = expr.get_span_pos();
            self.ctx.start_pos = start;
            self.ctx.end_pos = end;
        }
        self.ctx.cur_node = expr.id.clone();
        let result = self.walk_expr(&expr.node);
        if let Some(symbol_ref) = result {
            self.update_symbol_info_by_node(symbol_ref, expr);
        }

        result
    }

    #[inline]
    pub fn stmt(&mut self, stmt: &'ctx ast::NodeRef<ast::Stmt>) -> ResolvedResult {
        let (start, end) = stmt.get_span_pos();
        self.ctx.start_pos = start;
        self.ctx.end_pos = end;
        self.ctx.cur_node = stmt.id.clone();
        let result = self.walk_stmt(&stmt.node);
        if let Some(symbol_ref) = result {
            self.update_symbol_info_by_node(symbol_ref, stmt);
        }
        result
    }

    fn resolve_names(&mut self, names: &[ast::Node<String>], maybe_def: bool) -> ResolvedResult {
        let first_name = names.get(0)?;
        let cur_scope = *self.ctx.scopes.last().unwrap();

        let mut first_symbol =
            self.gs
                .look_up_symbol(&first_name.node, cur_scope, self.get_current_module_info());
        if first_symbol.is_none() {
            //maybe import package symbol
            let module_info = self.get_current_module_info().unwrap();

            let import_info = module_info.get_import_info(&first_name.node);
            if import_info.is_some() {
                first_symbol = self
                    .gs
                    .get_symbols()
                    .get_symbol_by_fully_qualified_name(&import_info.unwrap().fully_qualified_name)
            }
        }
        match first_symbol {
            Some(symbol_ref) => {
                let (start_pos, end_pos): Range = first_name.get_span_pos();
                let (def_start_pos, def_end_pos) =
                    self.gs.get_symbols().get_symbol(symbol_ref)?.get_range();

                // get an unresolved symbol
                if def_start_pos != start_pos || def_end_pos != end_pos {
                    let ast_id = first_name.id.clone();
                    let mut first_unresolved =
                        UnresolvedSymbol::new(first_name.node.clone(), start_pos, end_pos, None);
                    first_unresolved.def = Some(symbol_ref);
                    let first_unresolved_ref = self
                        .gs
                        .get_symbols_mut()
                        .alloc_unresolved_symbol(first_unresolved, &ast_id);
                    self.update_symbol_info_by_node(first_unresolved_ref, &first_name);
                    let cur_scope = *self.ctx.scopes.last().unwrap();
                    self.gs
                        .get_scopes_mut()
                        .add_ref_to_scope(cur_scope, first_unresolved_ref);
                }

                let mut parent_ty = self.ctx.node_ty_map.get(&first_name.id)?;

                for index in 1..names.len() {
                    let name = names.get(index).unwrap();
                    let def_symbol_ref = self.gs.get_symbols().get_type_attribute(
                        &parent_ty,
                        &name.node,
                        self.get_current_module_info(),
                    )?;

                    let (start_pos, end_pos): Range = name.get_span_pos();
                    let ast_id = name.id.clone();
                    let mut unresolved =
                        UnresolvedSymbol::new(name.node.clone(), start_pos, end_pos, None);
                    unresolved.def = Some(def_symbol_ref);
                    let unresolved_ref = self
                        .gs
                        .get_symbols_mut()
                        .alloc_unresolved_symbol(unresolved, &ast_id);

                    self.update_symbol_info_by_node(unresolved_ref, &name);
                    let cur_scope = *self.ctx.scopes.last().unwrap();
                    self.gs
                        .get_scopes_mut()
                        .add_ref_to_scope(cur_scope, unresolved_ref);

                    parent_ty = self.ctx.node_ty_map.get(&name.id)?;
                    if index == names.len() - 1 {
                        return Some(unresolved_ref);
                    }
                }

                Some(symbol_ref)
            }
            None => {
                if maybe_def {
                    let (start_pos, end_pos): Range = first_name.get_span_pos();
                    let ast_id = first_name.id.clone();
                    let first_value = self.gs.get_symbols_mut().alloc_value_symbol(
                        ValueSymbol::new(first_name.node.clone(), start_pos, end_pos, None, false),
                        &ast_id,
                    );
                    self.gs.get_scopes_mut().add_def_to_scope(
                        cur_scope,
                        first_name.node.clone(),
                        first_value,
                    );

                    self.update_symbol_info_by_node(first_value, first_name);

                    for index in 1..names.len() {
                        let name = names.get(index)?;
                        let (start_pos, end_pos): Range = name.get_span_pos();
                        let ast_id = name.id.clone();
                        let value = self.gs.get_symbols_mut().alloc_value_symbol(
                            ValueSymbol::new(name.node.clone(), start_pos, end_pos, None, false),
                            &ast_id,
                        );

                        self.gs.get_scopes_mut().add_def_to_scope(
                            cur_scope,
                            name.node.clone(),
                            value,
                        );

                        self.update_symbol_info_by_node(value, name);
                        if index == names.len() - 1 {
                            return Some(value);
                        }
                    }
                }
                None
            }
        }
    }

    #[inline]
    pub fn walk_identifier_expr(
        &mut self,
        identifier: &'ctx ast::NodeRef<ast::Identifier>,
    ) -> ResolvedResult {
        let symbol_ref = if let Some(identifier_symbol) = self
            .gs
            .get_symbols()
            .symbols_info
            .ast_id_map
            .get(&identifier.id)
        {
            *identifier_symbol
        } else {
            self.resolve_names(&identifier.node.names, self.ctx.maybe_def)?
        };

        self.update_symbol_info_by_node(symbol_ref, identifier);
        Some(symbol_ref)
    }

    pub fn walk_type_expr(
        &mut self,
        ty_node: Option<&'ctx ast::Node<ast::Type>>,
    ) -> ResolvedResult {
        if let Some(ty_node) = ty_node {
            match &ty_node.node {
                ast::Type::Any => {}
                ast::Type::Named(identifier) => {
                    self.walk_identifier(identifier);
                }
                ast::Type::Basic(_) => {}
                ast::Type::List(list_type) => {
                    self.walk_type_expr(list_type.inner_type.as_ref().map(|ty| ty.as_ref()));
                }
                ast::Type::Dict(dict_type) => {
                    self.walk_type_expr(dict_type.key_type.as_ref().map(|ty| ty.as_ref()));
                    self.walk_type_expr(dict_type.value_type.as_ref().map(|ty| ty.as_ref()));
                }
                ast::Type::Union(union_type) => {
                    for elem_ty in union_type.type_elements.iter() {
                        self.walk_type_expr(Some(elem_ty));
                    }
                }
                ast::Type::Literal(_) => {}
            }
        }
        None
    }

    pub fn do_arguments_symbol_resolve(
        &mut self,
        args: &'ctx [ast::NodeRef<ast::Expr>],
        kwargs: &'ctx [ast::NodeRef<ast::Keyword>],
    ) {
        for arg in args.iter() {
            self.expr(arg);
        }
        for kw in kwargs.iter() {
            if let Some(value) = &kw.node.value {
                self.expr(value);
            }
            let (start_pos, end_pos): Range = kw.get_span_pos();
            let value = self.gs.get_symbols_mut().alloc_value_symbol(
                ValueSymbol::new(kw.node.arg.node.get_name(), start_pos, end_pos, None, false),
                &kw.id,
            );
            self.update_symbol_info_by_node(value, kw);
        }
    }

    pub(crate) fn walk_config_entries(&mut self, entries: &'ctx [ast::NodeRef<ast::ConfigEntry>]) {
        let (start, end) = (self.ctx.start_pos.clone(), self.ctx.end_pos.clone());

        self.enter_local_scope(
            &self.ctx.current_filename.as_ref().unwrap().clone(),
            start,
            end,
        );
        let schema_symbol = self.ctx.current_schema_symbol.take();
        if let Some(owner) = schema_symbol {
            let cur_scope = self.ctx.scopes.last().unwrap();
            self.gs
                .get_scopes_mut()
                .set_owner_to_scope(*cur_scope, owner)
        }

        for entry in entries.iter() {
            if let Some(key) = &entry.node.key {
                self.ctx.maybe_def = true;
                self.expr(key);
                self.ctx.maybe_def = false;
            }
            self.expr(&entry.node.value);
        }
        self.leave_scope()
    }
}
