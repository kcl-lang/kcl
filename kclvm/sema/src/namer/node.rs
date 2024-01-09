use crate::core::package::ImportInfo;
use crate::core::symbol::{
    AttributeSymbol, RuleSymbol, SchemaSymbol, SymbolKind, SymbolRef, TypeAliasSymbol, ValueSymbol,
};

use super::Namer;
use kclvm_ast::ast;
use kclvm_ast::pos::GetPos;
use kclvm_ast::walker::MutSelfTypedResultWalker;
use kclvm_error::diagnostic::Range;

impl<'ctx> MutSelfTypedResultWalker<'ctx> for Namer<'ctx> {
    type Result = Option<Vec<SymbolRef>>;
    fn walk_module(&mut self, module: &'ctx ast::Module) -> Self::Result {
        let owner = *self.ctx.owner_symbols.last().unwrap();
        for stmt_node in module.body.iter() {
            let symbol_refs = self.walk_stmt(&stmt_node.node);

            if let Some(symbol_refs) = symbol_refs {
                for symbol_ref in symbol_refs {
                    let full_name = self
                        .gs
                        .get_symbols()
                        .get_fully_qualified_name(symbol_ref)
                        .unwrap();
                    let name = full_name.split(".").last().unwrap().to_string();

                    let package_symbol = self
                        .gs
                        .get_symbols_mut()
                        .packages
                        .get_mut(owner.get_id())
                        .unwrap();

                    if !package_symbol.members.contains_key(&name) {
                        package_symbol.members.insert(name, symbol_ref);
                    }
                }
            }
        }

        None
    }

    fn walk_expr_stmt(&mut self, _expr_stmt: &'ctx ast::ExprStmt) -> Self::Result {
        None
    }

    fn walk_unification_stmt(
        &mut self,
        unification_stmt: &'ctx ast::UnificationStmt,
    ) -> Self::Result {
        let (start_pos, end_pos): Range = unification_stmt.target.get_span_pos();
        let owner = self.ctx.owner_symbols.last().unwrap().clone();
        if unification_stmt.target.node.names.len() == 1 {
            let owner_fully_qualified_name = self
                .gs
                .get_symbols()
                .get_fully_qualified_name(owner)
                .unwrap();
            let value_name = unification_stmt.target.node.get_name();
            let value_fully_qualified_name = owner_fully_qualified_name + "." + &value_name;
            if !self
                .ctx
                .value_fully_qualified_name_set
                .contains(&value_fully_qualified_name)
            {
                let value_ref = self.gs.get_symbols_mut().alloc_value_symbol(
                    ValueSymbol::new(value_name, start_pos, end_pos, Some(owner), true),
                    self.ctx.get_node_key(&unification_stmt.target.id),
                );
                self.ctx
                    .value_fully_qualified_name_set
                    .insert(value_fully_qualified_name);
                Some(vec![value_ref])
            } else {
                None
            }
        } else {
            None
        }
    }

    fn walk_type_alias_stmt(&mut self, type_alias_stmt: &'ctx ast::TypeAliasStmt) -> Self::Result {
        let (start_pos, end_pos): Range = type_alias_stmt.type_name.get_span_pos();
        let owner = self.ctx.owner_symbols.last().unwrap().clone();
        let type_alias_ref = self.gs.get_symbols_mut().alloc_type_alias_symbol(
            TypeAliasSymbol::new(
                type_alias_stmt.type_name.node.get_name(),
                start_pos,
                end_pos,
                owner,
            ),
            self.ctx.get_node_key(&type_alias_stmt.type_name.id),
        );
        Some(vec![type_alias_ref])
    }

    fn walk_assign_stmt(&mut self, assign_stmt: &'ctx ast::AssignStmt) -> Self::Result {
        let mut value_symbols = vec![];
        for target in assign_stmt.targets.iter() {
            let (start_pos, end_pos): Range = target.get_span_pos();
            let owner = self.ctx.owner_symbols.last().unwrap().clone();
            if target.node.names.len() == 1 {
                let owner_fully_qualified_name = self
                    .gs
                    .get_symbols()
                    .get_fully_qualified_name(owner)
                    .unwrap();
                let value_name = target.node.get_name();
                let value_fully_qualified_name = owner_fully_qualified_name + "." + &value_name;
                if !self
                    .ctx
                    .value_fully_qualified_name_set
                    .contains(&value_fully_qualified_name)
                {
                    let value_ref = self.gs.get_symbols_mut().alloc_value_symbol(
                        ValueSymbol::new(value_name, start_pos, end_pos, Some(owner), true),
                        self.ctx.get_node_key(&target.id),
                    );
                    self.ctx
                        .value_fully_qualified_name_set
                        .insert(value_fully_qualified_name);
                    value_symbols.push(value_ref)
                }
            }
        }
        Some(value_symbols)
    }

    fn walk_aug_assign_stmt(&mut self, _aug_assign_stmt: &'ctx ast::AugAssignStmt) -> Self::Result {
        None
    }

    fn walk_assert_stmt(&mut self, _assert_stmt: &'ctx ast::AssertStmt) -> Self::Result {
        None
    }

    fn walk_if_stmt(&mut self, if_stmt: &'ctx ast::IfStmt) -> Self::Result {
        let mut all_symbols = vec![];
        for stmt in if_stmt.body.iter() {
            let mut symbols = self.walk_stmt(&stmt.node);
            if let Some(symbols) = &mut symbols {
                all_symbols.append(symbols);
            }
        }
        for stmt in if_stmt.orelse.iter() {
            let mut symbols = self.walk_stmt(&stmt.node);
            if let Some(symbols) = &mut symbols {
                all_symbols.append(symbols);
            }
        }
        Some(all_symbols)
    }

    fn walk_import_stmt(&mut self, import_stmt: &'ctx ast::ImportStmt) -> Self::Result {
        self.ctx
            .current_module_info
            .as_mut()
            .unwrap()
            .add_import_info(ImportInfo::new(
                import_stmt.name.clone(),
                import_stmt.path.node.clone(),
            ));

        None
    }

    fn walk_schema_stmt(&mut self, schema_stmt: &'ctx ast::SchemaStmt) -> Self::Result {
        let (start_pos, end_pos): Range = schema_stmt.name.get_span_pos();
        let owner = self.ctx.owner_symbols.last().unwrap();
        let shcema_ref = self.gs.get_symbols_mut().alloc_schema_symbol(
            SchemaSymbol::new(schema_stmt.name.node.clone(), start_pos, end_pos, *owner),
            self.ctx.get_node_key(&schema_stmt.name.id),
        );
        self.ctx.owner_symbols.push(shcema_ref);

        for stmt in schema_stmt.body.iter() {
            let symbol_refs = self.walk_stmt(&stmt.node);
            if let Some(symbol_refs) = symbol_refs {
                for symbol_ref in symbol_refs {
                    if matches!(&symbol_ref.get_kind(), SymbolKind::Attribute) {
                        let full_attribute_name = self
                            .gs
                            .get_symbols()
                            .get_fully_qualified_name(symbol_ref)
                            .unwrap();
                        self.ctx
                            .value_fully_qualified_name_set
                            .insert(full_attribute_name.clone());
                        let attribute_name =
                            full_attribute_name.split(".").last().unwrap().to_string();

                        let schema_symbol = self
                            .gs
                            .get_symbols_mut()
                            .schemas
                            .get_mut(shcema_ref.get_id())
                            .unwrap();
                        if !schema_symbol.attributes.contains_key(&attribute_name) {
                            schema_symbol.attributes.insert(attribute_name, symbol_ref);
                        }
                    }
                }
            }
        }
        self.ctx.owner_symbols.pop();
        Some(vec![shcema_ref])
    }

    fn walk_rule_stmt(&mut self, rule_stmt: &'ctx ast::RuleStmt) -> Self::Result {
        let (start_pos, end_pos): Range = rule_stmt.name.get_span_pos();
        let owner = self.ctx.owner_symbols.last().unwrap().clone();
        let rule_ref = self.gs.get_symbols_mut().alloc_rule_symbol(
            RuleSymbol::new(rule_stmt.name.node.clone(), start_pos, end_pos, owner),
            self.ctx.get_node_key(&rule_stmt.name.id),
        );
        Some(vec![rule_ref])
    }

    fn walk_quant_expr(&mut self, _quant_expr: &'ctx ast::QuantExpr) -> Self::Result {
        None
    }

    fn walk_schema_attr(&mut self, schema_attr: &'ctx ast::SchemaAttr) -> Self::Result {
        let (start_pos, end_pos): Range = schema_attr.name.get_span_pos();
        let owner = self.ctx.owner_symbols.last().unwrap().clone();
        let attribute_ref = self.gs.get_symbols_mut().alloc_attribute_symbol(
            AttributeSymbol::new(schema_attr.name.node.clone(), start_pos, end_pos, owner),
            self.ctx.get_node_key(&schema_attr.name.id),
        );
        Some(vec![attribute_ref])
    }

    /// <body> if <cond> else <orelse> -> sup([body, orelse])
    fn walk_if_expr(&mut self, _if_expr: &'ctx ast::IfExpr) -> Self::Result {
        None
    }

    fn walk_unary_expr(&mut self, _unary_expr: &'ctx ast::UnaryExpr) -> Self::Result {
        None
    }

    fn walk_binary_expr(&mut self, _binary_expr: &'ctx ast::BinaryExpr) -> Self::Result {
        None
    }

    fn walk_selector_expr(&mut self, _selector_expr: &'ctx ast::SelectorExpr) -> Self::Result {
        None
    }

    fn walk_call_expr(&mut self, _call_expr: &'ctx ast::CallExpr) -> Self::Result {
        None
    }

    fn walk_subscript(&mut self, _subscript: &'ctx ast::Subscript) -> Self::Result {
        None
    }

    fn walk_paren_expr(&mut self, _paren_expr: &'ctx ast::ParenExpr) -> Self::Result {
        None
    }

    fn walk_list_expr(&mut self, _list_expr: &'ctx ast::ListExpr) -> Self::Result {
        None
    }

    fn walk_list_comp(&mut self, _list_comp: &'ctx ast::ListComp) -> Self::Result {
        None
    }

    fn walk_dict_comp(&mut self, _dict_comp: &'ctx ast::DictComp) -> Self::Result {
        None
    }

    fn walk_list_if_item_expr(
        &mut self,
        _list_if_item_expr: &'ctx ast::ListIfItemExpr,
    ) -> Self::Result {
        None
    }

    fn walk_starred_expr(&mut self, _starred_expr: &'ctx ast::StarredExpr) -> Self::Result {
        None
    }

    fn walk_config_if_entry_expr(
        &mut self,
        _config_if_entry_expr: &'ctx ast::ConfigIfEntryExpr,
    ) -> Self::Result {
        None
    }

    fn walk_comp_clause(&mut self, _comp_clause: &'ctx ast::CompClause) -> Self::Result {
        None
    }

    fn walk_schema_expr(&mut self, _schema_expr: &'ctx ast::SchemaExpr) -> Self::Result {
        None
    }

    fn walk_config_expr(&mut self, _config_expr: &'ctx ast::ConfigExpr) -> Self::Result {
        None
    }

    fn walk_check_expr(&mut self, _check_expr: &'ctx ast::CheckExpr) -> Self::Result {
        None
    }

    fn walk_lambda_expr(&mut self, _lambda_expr: &'ctx ast::LambdaExpr) -> Self::Result {
        None
    }

    fn walk_keyword(&mut self, _keyword: &'ctx ast::Keyword) -> Self::Result {
        None
    }

    fn walk_arguments(&mut self, _arguments: &'ctx ast::Arguments) -> Self::Result {
        None
    }

    fn walk_compare(&mut self, _compare: &'ctx ast::Compare) -> Self::Result {
        None
    }

    fn walk_identifier(&mut self, _identifier: &'ctx ast::Identifier) -> Self::Result {
        None
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

    fn walk_joined_string(&mut self, _joined_string: &'ctx ast::JoinedString) -> Self::Result {
        None
    }

    fn walk_formatted_value(
        &mut self,
        _formatted_value: &'ctx ast::FormattedValue,
    ) -> Self::Result {
        None
    }

    fn walk_comment(&mut self, _comment: &'ctx ast::Comment) -> Self::Result {
        None
    }

    fn walk_missing_expr(&mut self, _missing_expr: &'ctx ast::MissingExpr) -> Self::Result {
        None
    }
}
