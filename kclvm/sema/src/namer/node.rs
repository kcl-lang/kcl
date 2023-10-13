use crate::core::package::{ImportInfo, ModuleInfo};
use crate::core::symbol::{
    AttributeSymbol, RuleSymbol, SchemaSymbol, SymbolKind, SymbolRef, TypeAliasSymbol,
    UnresolvedSymbol,
};

use super::Namer;
use kclvm_ast::ast;
use kclvm_ast::walker::MutSelfTypedResultWalker;
use kclvm_error::Position;

fn get_node_position<T>(node: &ast::Node<T>) -> (Position, Position) {
    (
        Position {
            filename: node.filename.clone(),
            line: node.line,
            column: Some(node.column),
        },
        Position {
            filename: node.filename.clone(),
            line: node.end_line,
            column: Some(node.end_column),
        },
    )
}

impl<'ctx> MutSelfTypedResultWalker<'ctx> for Namer<'ctx> {
    type Result = SymbolRef;
    fn walk_module(&mut self, module: &'ctx ast::Module) -> Self::Result {
        let owner = *self.ctx.owner_symbols.last().unwrap();
        for stmt_node in module.body.iter() {
            let symbol_ref = self.walk_stmt(&stmt_node.node);

            match symbol_ref.get_kind() {
                SymbolKind::Dummy => continue,
                _ => {
                    let member_name = self.gs.get_symbols().get_fully_qualified_name(symbol_ref);
                    self.gs
                        .get_symbols_mut()
                        .packages
                        .get_mut(owner.get_id())
                        .unwrap()
                        .members
                        .insert(member_name, symbol_ref);
                }
            }
        }
        self.ctx
            .current_package_info
            .as_mut()
            .unwrap()
            .add_module_info(ModuleInfo::new(module.filename.clone()));

        SymbolRef::dummy_symbol()
    }

    fn walk_expr_stmt(&mut self, _expr_stmt: &'ctx ast::ExprStmt) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_unification_stmt(
        &mut self,
        _unification_stmt: &'ctx ast::UnificationStmt,
    ) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_type_alias_stmt(&mut self, type_alias_stmt: &'ctx ast::TypeAliasStmt) -> Self::Result {
        let (start_pos, end_pos) = get_node_position(&type_alias_stmt.type_name);
        let owner = self.ctx.owner_symbols.last().unwrap().clone();
        let type_alias_ref =
            self.gs
                .get_symbols_mut()
                .alloc_type_alias_symbol(TypeAliasSymbol::new(
                    type_alias_stmt.type_name.node.get_name(),
                    start_pos,
                    end_pos,
                    owner,
                ));
        type_alias_ref
    }

    fn walk_assign_stmt(&mut self, _assign_stmt: &'ctx ast::AssignStmt) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_aug_assign_stmt(&mut self, _aug_assign_stmt: &'ctx ast::AugAssignStmt) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_assert_stmt(&mut self, _assert_stmt: &'ctx ast::AssertStmt) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_if_stmt(&mut self, _if_stmt: &'ctx ast::IfStmt) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_import_stmt(&mut self, import_stmt: &'ctx ast::ImportStmt) -> Self::Result {
        self.ctx
            .current_package_info
            .as_mut()
            .unwrap()
            .add_import_info(ImportInfo::new(
                import_stmt.name.clone(),
                import_stmt.path.clone(),
            ));

        SymbolRef::dummy_symbol()
    }

    fn walk_schema_stmt(&mut self, schema_stmt: &'ctx ast::SchemaStmt) -> Self::Result {
        let (start_pos, end_pos) = get_node_position(&schema_stmt.name);
        let owner = self.ctx.owner_symbols.last().unwrap();

        let shcema_ref = self
            .gs
            .get_symbols_mut()
            .alloc_schema_symbol(SchemaSymbol::new(
                schema_stmt.name.node.clone(),
                start_pos,
                end_pos,
                *owner,
            ));
        self.ctx.owner_symbols.push(shcema_ref);

        self.gs
            .get_symbols_mut()
            .schemas
            .get_mut(shcema_ref.get_id())
            .unwrap()
            .parent_schema = schema_stmt.parent_name.as_ref().map(|name| {
            let (start_pos, end_pos) = get_node_position(&name);
            self.gs
                .get_symbols_mut()
                .alloc_unresolved_symbol(UnresolvedSymbol::new(
                    name.node.get_name(),
                    start_pos,
                    end_pos,
                    shcema_ref,
                ))
        });

        for mixin in schema_stmt.mixins.iter() {
            let (start_pos, end_pos) = get_node_position(&schema_stmt.name);
            let mixin_ref =
                self.gs
                    .get_symbols_mut()
                    .alloc_unresolved_symbol(UnresolvedSymbol::new(
                        mixin.node.get_name(),
                        start_pos,
                        end_pos,
                        shcema_ref,
                    ));
            self.gs
                .get_symbols_mut()
                .schemas
                .get_mut(shcema_ref.get_id())
                .unwrap()
                .mixins
                .push(mixin_ref);
        }

        for stmt in schema_stmt.body.iter() {
            let symbol_ref = self.walk_stmt(&stmt.node);
            if matches!(&symbol_ref.get_kind(), SymbolKind::Attribute) {
                let attribut_name = self.gs.get_symbols().get_fully_qualified_name(symbol_ref);
                let schema_symbol = self
                    .gs
                    .get_symbols_mut()
                    .schemas
                    .get_mut(shcema_ref.get_id())
                    .unwrap();

                schema_symbol.attributes.insert(attribut_name, symbol_ref);
            }
        }
        self.ctx.owner_symbols.pop();
        shcema_ref
    }

    fn walk_rule_stmt(&mut self, rule_stmt: &'ctx ast::RuleStmt) -> Self::Result {
        let (start_pos, end_pos) = get_node_position(&rule_stmt.name);
        let owner = self.ctx.owner_symbols.last().unwrap().clone();
        let attribute_ref = self.gs.get_symbols_mut().alloc_rule_symbol(RuleSymbol::new(
            rule_stmt.name.node.clone(),
            start_pos,
            end_pos,
            owner,
        ));
        attribute_ref
    }

    fn walk_quant_expr(&mut self, _quant_expr: &'ctx ast::QuantExpr) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_schema_attr(&mut self, schema_attr: &'ctx ast::SchemaAttr) -> Self::Result {
        let (start_pos, end_pos) = get_node_position(&schema_attr.name);
        let owner = self.ctx.owner_symbols.last().unwrap().clone();
        let attribute_ref = self
            .gs
            .get_symbols_mut()
            .alloc_attribute_symbol(AttributeSymbol::new(
                schema_attr.name.node.clone(),
                start_pos,
                end_pos,
                owner,
            ));
        attribute_ref
    }

    /// <body> if <cond> else <orelse> -> sup([body, orelse])
    fn walk_if_expr(&mut self, _if_expr: &'ctx ast::IfExpr) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_unary_expr(&mut self, _unary_expr: &'ctx ast::UnaryExpr) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_binary_expr(&mut self, _binary_expr: &'ctx ast::BinaryExpr) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_selector_expr(&mut self, _selector_expr: &'ctx ast::SelectorExpr) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_call_expr(&mut self, _call_expr: &'ctx ast::CallExpr) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_subscript(&mut self, _subscript: &'ctx ast::Subscript) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_paren_expr(&mut self, _paren_expr: &'ctx ast::ParenExpr) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_list_expr(&mut self, _list_expr: &'ctx ast::ListExpr) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_list_comp(&mut self, _list_comp: &'ctx ast::ListComp) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_dict_comp(&mut self, _dict_comp: &'ctx ast::DictComp) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_list_if_item_expr(
        &mut self,
        _list_if_item_expr: &'ctx ast::ListIfItemExpr,
    ) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_starred_expr(&mut self, _starred_expr: &'ctx ast::StarredExpr) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_config_if_entry_expr(
        &mut self,
        _config_if_entry_expr: &'ctx ast::ConfigIfEntryExpr,
    ) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_comp_clause(&mut self, _comp_clause: &'ctx ast::CompClause) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_schema_expr(&mut self, _schema_expr: &'ctx ast::SchemaExpr) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_config_expr(&mut self, _config_expr: &'ctx ast::ConfigExpr) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_check_expr(&mut self, _check_expr: &'ctx ast::CheckExpr) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_lambda_expr(&mut self, _lambda_expr: &'ctx ast::LambdaExpr) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_keyword(&mut self, _keyword: &'ctx ast::Keyword) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_arguments(&mut self, _arguments: &'ctx ast::Arguments) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_compare(&mut self, _compare: &'ctx ast::Compare) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_identifier(&mut self, _identifier: &'ctx ast::Identifier) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_number_lit(&mut self, _number_lit: &'ctx ast::NumberLit) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_string_lit(&mut self, _string_lit: &'ctx ast::StringLit) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_name_constant_lit(
        &mut self,
        _name_constant_lit: &'ctx ast::NameConstantLit,
    ) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_joined_string(&mut self, _joined_string: &'ctx ast::JoinedString) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_formatted_value(
        &mut self,
        _formatted_value: &'ctx ast::FormattedValue,
    ) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_comment(&mut self, _comment: &'ctx ast::Comment) -> Self::Result {
        SymbolRef::dummy_symbol()
    }

    fn walk_missing_expr(&mut self, _missing_expr: &'ctx ast::MissingExpr) -> Self::Result {
        SymbolRef::dummy_symbol()
    }
}
