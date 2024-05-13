use kclvm_ast::{
    ast::{self},
    walker::MutSelfMutWalker,
};

use kclvm_ast::walk_if_mut;
use kclvm_ast::walk_list_mut;

/// `AstNodeMover` will move the AST node by offset
pub struct AstNodeMover {
    pub line_offset: usize,
}

impl<'ctx> MutSelfMutWalker<'ctx> for AstNodeMover {
    fn walk_expr_stmt(&mut self, expr_stmt: &'ctx mut ast::ExprStmt) {
        for expr in expr_stmt.exprs.iter_mut() {
            expr.line += self.line_offset as u64;
            expr.end_line += self.line_offset as u64;
        }

        for expr in expr_stmt.exprs.iter_mut() {
            self.walk_expr(&mut expr.node)
        }
    }
    fn walk_type_alias_stmt(&mut self, type_alias_stmt: &'ctx mut ast::TypeAliasStmt) {
        type_alias_stmt.type_name.line += self.line_offset as u64;
        type_alias_stmt.type_name.end_line += self.line_offset as u64;

        type_alias_stmt.ty.line += self.line_offset as u64;
        type_alias_stmt.ty.end_line += self.line_offset as u64;

        self.walk_identifier(&mut type_alias_stmt.type_name.node);
        self.walk_type(&mut type_alias_stmt.ty.node);
    }
    fn walk_unification_stmt(&mut self, unification_stmt: &'ctx mut ast::UnificationStmt) {
        unification_stmt.target.line += self.line_offset as u64;
        unification_stmt.target.end_line += self.line_offset as u64;

        unification_stmt.value.line += self.line_offset as u64;
        unification_stmt.value.end_line += self.line_offset as u64;

        self.walk_identifier(&mut unification_stmt.target.node);
        self.walk_schema_expr(&mut unification_stmt.value.node);
    }
    fn walk_assign_stmt(&mut self, assign_stmt: &'ctx mut ast::AssignStmt) {
        for target in assign_stmt.targets.iter_mut() {
            target.line += self.line_offset as u64;
            target.end_line += self.line_offset as u64;
        }

        assign_stmt.value.line += self.line_offset as u64;
        assign_stmt.value.end_line += self.line_offset as u64;

        match assign_stmt.ty.as_deref_mut() {
            Some(v) => {
                v.line += self.line_offset as u64;
                v.end_line += self.line_offset as u64;
            }
            None => (),
        }

        for target in assign_stmt.targets.iter_mut() {
            self.walk_identifier(&mut target.node)
        }
        self.walk_expr(&mut assign_stmt.value.node);
        walk_if_mut!(self, walk_type, assign_stmt.ty)
    }
    fn walk_aug_assign_stmt(&mut self, aug_assign_stmt: &'ctx mut ast::AugAssignStmt) {
        aug_assign_stmt.target.line += self.line_offset as u64;
        aug_assign_stmt.target.end_line += self.line_offset as u64;

        aug_assign_stmt.value.line += self.line_offset as u64;
        aug_assign_stmt.value.end_line += self.line_offset as u64;

        self.walk_identifier(&mut aug_assign_stmt.target.node);
        self.walk_expr(&mut aug_assign_stmt.value.node);
    }
    fn walk_assert_stmt(&mut self, assert_stmt: &'ctx mut ast::AssertStmt) {
        assert_stmt.test.line += self.line_offset as u64;
        assert_stmt.test.end_line += self.line_offset as u64;

        match assert_stmt.if_cond.as_deref_mut() {
            Some(v) => {
                v.line += self.line_offset as u64;
                v.end_line += self.line_offset as u64;
            }
            None => (),
        }

        match assert_stmt.msg.as_deref_mut() {
            Some(v) => {
                v.line += self.line_offset as u64;
                v.end_line += self.line_offset as u64;
            }
            None => (),
        }

        self.walk_expr(&mut assert_stmt.test.node);
        walk_if_mut!(self, walk_expr, assert_stmt.if_cond);
        walk_if_mut!(self, walk_expr, assert_stmt.msg);
    }
    fn walk_if_stmt(&mut self, if_stmt: &'ctx mut ast::IfStmt) {
        if_stmt.cond.line += self.line_offset as u64;
        if_stmt.cond.end_line += self.line_offset as u64;

        for stmt in if_stmt.body.iter_mut() {
            stmt.line += self.line_offset as u64;
            stmt.end_line += self.line_offset as u64;
        }

        for stmt in if_stmt.orelse.iter_mut() {
            stmt.line += self.line_offset as u64;
            stmt.end_line += self.line_offset as u64;
        }

        self.walk_expr(&mut if_stmt.cond.node);
        walk_list_mut!(self, walk_stmt, if_stmt.body);
        walk_list_mut!(self, walk_stmt, if_stmt.orelse);
    }
    fn walk_import_stmt(&mut self, _import_stmt: &'ctx mut ast::ImportStmt) {
        // Nothing to do
    }
    fn walk_schema_attr(&mut self, schema_attr: &'ctx mut ast::SchemaAttr) {
        schema_attr.name.line += self.line_offset as u64;
        schema_attr.name.end_line += self.line_offset as u64;

        match schema_attr.value.as_deref_mut() {
            Some(v) => {
                v.line += self.line_offset as u64;
                v.end_line += self.line_offset as u64;
            }
            None => (),
        }

        schema_attr.decorators.iter_mut().for_each(|d| {
            d.line += self.line_offset as u64;
            d.end_line += self.line_offset as u64;
        });

        schema_attr.ty.line += self.line_offset as u64;
        schema_attr.ty.end_line += self.line_offset as u64;

        walk_list_mut!(self, walk_call_expr, schema_attr.decorators);
        walk_if_mut!(self, walk_expr, schema_attr.value);
        self.walk_type(&mut schema_attr.ty.node);
    }

    fn walk_type(&mut self, ty: &'ctx mut ast::Type) {
        match ty {
            ast::Type::Named(id) => self.walk_identifier(id),
            ast::Type::List(list_ty) => {
                if let Some(ty) = &mut list_ty.inner_type {
                    ty.line += self.line_offset as u64;
                    ty.end_line += self.line_offset as u64;
                    self.walk_type(&mut ty.node)
                }
            }
            ast::Type::Dict(dict_ty) => {
                if let Some(ty) = &mut dict_ty.key_type {
                    ty.line += self.line_offset as u64;
                    ty.end_line += self.line_offset as u64;
                    self.walk_type(&mut ty.node)
                }
                if let Some(ty) = &mut dict_ty.value_type {
                    ty.line += self.line_offset as u64;
                    ty.end_line += self.line_offset as u64;
                    self.walk_type(&mut ty.node)
                }
            }
            ast::Type::Union(union_ty) => {
                union_ty.type_elements.iter_mut().for_each(|ty| {
                    ty.line += self.line_offset as u64;
                    ty.end_line += self.line_offset as u64;
                    self.walk_type(&mut ty.node)
                });
            }
            _ => {}
        }
    }
    fn walk_schema_stmt(&mut self, schema_stmt: &'ctx mut ast::SchemaStmt) {
        schema_stmt.name.line += self.line_offset as u64;
        schema_stmt.name.end_line += self.line_offset as u64;

        match schema_stmt.parent_name.as_deref_mut() {
            Some(v) => {
                v.line += self.line_offset as u64;
                v.end_line += self.line_offset as u64;
            }
            None => (),
        }

        match schema_stmt.for_host_name.as_deref_mut() {
            Some(v) => {
                v.line += self.line_offset as u64;
                v.end_line += self.line_offset as u64;
            }
            None => (),
        }

        for arg in schema_stmt.args.iter_mut() {
            arg.line += self.line_offset as u64;
            arg.end_line += self.line_offset as u64;
        }

        if let Some(schema_index_signature) = schema_stmt.index_signature.as_deref_mut() {
            let value = &mut schema_index_signature.node.value;
            match value.as_deref_mut() {
                Some(v) => {
                    v.line += self.line_offset as u64;
                    v.end_line += self.line_offset as u64;
                }
                None => (),
            }
        }

        schema_stmt.mixins.iter_mut().for_each(|m| {
            m.line += self.line_offset as u64;
            m.end_line += self.line_offset as u64;
        });

        schema_stmt.decorators.iter_mut().for_each(|d| {
            d.line += self.line_offset as u64;
            d.end_line += self.line_offset as u64;
        });

        schema_stmt.checks.iter_mut().for_each(|c| {
            c.line += self.line_offset as u64;
            c.end_line += self.line_offset as u64;
        });

        schema_stmt.body.iter_mut().for_each(|s| {
            s.line += self.line_offset as u64;
            s.end_line += self.line_offset as u64;
        });

        walk_if_mut!(self, walk_identifier, schema_stmt.parent_name);
        walk_if_mut!(self, walk_identifier, schema_stmt.for_host_name);
        walk_if_mut!(self, walk_arguments, schema_stmt.args);
        if let Some(schema_index_signature) = schema_stmt.index_signature.as_deref_mut() {
            let value = &mut schema_index_signature.node.value;
            walk_if_mut!(self, walk_expr, value);
        }
        walk_list_mut!(self, walk_identifier, schema_stmt.mixins);
        walk_list_mut!(self, walk_call_expr, schema_stmt.decorators);
        walk_list_mut!(self, walk_check_expr, schema_stmt.checks);
        walk_list_mut!(self, walk_stmt, schema_stmt.body);
    }
    fn walk_rule_stmt(&mut self, rule_stmt: &'ctx mut ast::RuleStmt) {
        rule_stmt.name.line += self.line_offset as u64;
        rule_stmt.name.end_line += self.line_offset as u64;

        rule_stmt.parent_rules.iter_mut().for_each(|p| {
            p.line += self.line_offset as u64;
            p.end_line += self.line_offset as u64;
        });

        rule_stmt.decorators.iter_mut().for_each(|d| {
            d.line += self.line_offset as u64;
            d.end_line += self.line_offset as u64;
        });

        rule_stmt.checks.iter_mut().for_each(|c| {
            c.line += self.line_offset as u64;
            c.end_line += self.line_offset as u64;
        });

        rule_stmt.args.iter_mut().for_each(|a| {
            a.line += self.line_offset as u64;
            a.end_line += self.line_offset as u64;
        });

        match rule_stmt.for_host_name.as_deref_mut() {
            Some(v) => {
                v.line += self.line_offset as u64;
                v.end_line += self.line_offset as u64;
            }
            None => (),
        }

        walk_list_mut!(self, walk_identifier, rule_stmt.parent_rules);
        walk_list_mut!(self, walk_call_expr, rule_stmt.decorators);
        walk_list_mut!(self, walk_check_expr, rule_stmt.checks);
        walk_if_mut!(self, walk_arguments, rule_stmt.args);
        walk_if_mut!(self, walk_identifier, rule_stmt.for_host_name);
    }
    fn walk_quant_expr(&mut self, quant_expr: &'ctx mut ast::QuantExpr) {
        quant_expr.target.line += self.line_offset as u64;
        quant_expr.target.end_line += self.line_offset as u64;

        quant_expr.variables.iter_mut().for_each(|v| {
            v.line += self.line_offset as u64;
            v.end_line += self.line_offset as u64;
        });

        quant_expr.test.line += self.line_offset as u64;
        quant_expr.test.end_line += self.line_offset as u64;

        match quant_expr.if_cond.as_deref_mut() {
            Some(v) => {
                v.line += self.line_offset as u64;
                v.end_line += self.line_offset as u64;
            }
            None => (),
        }

        self.walk_expr(&mut quant_expr.target.node);
        walk_list_mut!(self, walk_identifier, quant_expr.variables);
        self.walk_expr(&mut quant_expr.test.node);
        walk_if_mut!(self, walk_expr, quant_expr.if_cond);
    }
    fn walk_if_expr(&mut self, if_expr: &'ctx mut ast::IfExpr) {
        if_expr.cond.line += self.line_offset as u64;
        if_expr.cond.end_line += self.line_offset as u64;

        if_expr.body.line += self.line_offset as u64;
        if_expr.body.end_line += self.line_offset as u64;

        if_expr.orelse.line += self.line_offset as u64;
        if_expr.orelse.end_line += self.line_offset as u64;

        self.walk_expr(&mut if_expr.cond.node);
        self.walk_expr(&mut if_expr.body.node);
        self.walk_expr(&mut if_expr.orelse.node);
    }
    fn walk_unary_expr(&mut self, unary_expr: &'ctx mut ast::UnaryExpr) {
        unary_expr.operand.line += self.line_offset as u64;
        unary_expr.operand.end_line += self.line_offset as u64;

        self.walk_expr(&mut unary_expr.operand.node);
    }
    fn walk_binary_expr(&mut self, binary_expr: &'ctx mut ast::BinaryExpr) {
        binary_expr.left.line += self.line_offset as u64;
        binary_expr.left.end_line += self.line_offset as u64;

        self.walk_expr(&mut binary_expr.left.node);
        self.walk_expr(&mut binary_expr.right.node);
    }
    fn walk_selector_expr(&mut self, selector_expr: &'ctx mut ast::SelectorExpr) {
        selector_expr.value.line += self.line_offset as u64;
        selector_expr.value.end_line += self.line_offset as u64;

        self.walk_expr(&mut selector_expr.value.node);
        self.walk_identifier(&mut selector_expr.attr.node);
    }
    fn walk_call_expr(&mut self, call_expr: &'ctx mut ast::CallExpr) {
        call_expr.func.line += self.line_offset as u64;
        call_expr.func.end_line += self.line_offset as u64;

        call_expr.args.iter_mut().for_each(|a| {
            a.line += self.line_offset as u64;
            a.end_line += self.line_offset as u64;
        });

        call_expr.keywords.iter_mut().for_each(|k| {
            k.line += self.line_offset as u64;
            k.end_line += self.line_offset as u64;
        });

        self.walk_expr(&mut call_expr.func.node);
        walk_list_mut!(self, walk_expr, call_expr.args);
        walk_list_mut!(self, walk_keyword, call_expr.keywords);
    }
    fn walk_subscript(&mut self, subscript: &'ctx mut ast::Subscript) {
        subscript.value.line += self.line_offset as u64;
        subscript.value.end_line += self.line_offset as u64;

        match subscript.index.as_deref_mut() {
            Some(v) => {
                v.line += self.line_offset as u64;
                v.end_line += self.line_offset as u64;
            }
            None => (),
        }

        match subscript.lower.as_deref_mut() {
            Some(v) => {
                v.line += self.line_offset as u64;
                v.end_line += self.line_offset as u64;
            }
            None => (),
        }

        match subscript.upper.as_deref_mut() {
            Some(v) => {
                v.line += self.line_offset as u64;
                v.end_line += self.line_offset as u64;
            }
            None => (),
        }

        match subscript.step.as_deref_mut() {
            Some(v) => {
                v.line += self.line_offset as u64;
                v.end_line += self.line_offset as u64;
            }
            None => (),
        }

        self.walk_expr(&mut subscript.value.node);
        walk_if_mut!(self, walk_expr, subscript.index);
        walk_if_mut!(self, walk_expr, subscript.lower);
        walk_if_mut!(self, walk_expr, subscript.upper);
        walk_if_mut!(self, walk_expr, subscript.step);
    }
    fn walk_paren_expr(&mut self, paren_expr: &'ctx mut ast::ParenExpr) {
        paren_expr.expr.line += self.line_offset as u64;
        paren_expr.expr.end_line += self.line_offset as u64;

        self.walk_expr(&mut paren_expr.expr.node);
    }
    fn walk_list_expr(&mut self, list_expr: &'ctx mut ast::ListExpr) {
        list_expr.elts.iter_mut().for_each(|e| {
            e.line += self.line_offset as u64;
            e.end_line += self.line_offset as u64;
        });
        walk_list_mut!(self, walk_expr, list_expr.elts);
    }
    fn walk_list_comp(&mut self, list_comp: &'ctx mut ast::ListComp) {
        list_comp.elt.line += self.line_offset as u64;
        list_comp.elt.end_line += self.line_offset as u64;

        list_comp.generators.iter_mut().for_each(|g| {
            g.line += self.line_offset as u64;
            g.end_line += self.line_offset as u64;
        });

        self.walk_expr(&mut list_comp.elt.node);
        walk_list_mut!(self, walk_comp_clause, list_comp.generators);
    }
    fn walk_list_if_item_expr(&mut self, list_if_item_expr: &'ctx mut ast::ListIfItemExpr) {
        list_if_item_expr.if_cond.line += self.line_offset as u64;
        list_if_item_expr.if_cond.end_line += self.line_offset as u64;

        list_if_item_expr.exprs.iter_mut().for_each(|e| {
            e.line += self.line_offset as u64;
            e.end_line += self.line_offset as u64;
        });

        match list_if_item_expr.orelse.as_deref_mut() {
            Some(v) => {
                v.line += self.line_offset as u64;
                v.end_line += self.line_offset as u64;
            }
            None => (),
        }

        self.walk_expr(&mut list_if_item_expr.if_cond.node);
        walk_list_mut!(self, walk_expr, list_if_item_expr.exprs);
        walk_if_mut!(self, walk_expr, list_if_item_expr.orelse);
    }
    fn walk_starred_expr(&mut self, starred_expr: &'ctx mut ast::StarredExpr) {
        starred_expr.value.line += self.line_offset as u64;
        starred_expr.value.end_line += self.line_offset as u64;
        self.walk_expr(&mut starred_expr.value.node);
    }
    fn walk_dict_comp(&mut self, dict_comp: &'ctx mut ast::DictComp) {
        if let Some(key) = &mut dict_comp.entry.key {
            key.line += self.line_offset as u64;
            key.end_line += self.line_offset as u64;
        }

        dict_comp.entry.value.line += self.line_offset as u64;
        dict_comp.entry.value.end_line += self.line_offset as u64;

        dict_comp.generators.iter_mut().for_each(|g| {
            g.line += self.line_offset as u64;
            g.end_line += self.line_offset as u64;
        });

        if let Some(key) = &mut dict_comp.entry.key {
            self.walk_expr(&mut key.node);
        }
        self.walk_expr(&mut dict_comp.entry.value.node);
        walk_list_mut!(self, walk_comp_clause, dict_comp.generators);
    }
    fn walk_config_if_entry_expr(
        &mut self,
        config_if_entry_expr: &'ctx mut ast::ConfigIfEntryExpr,
    ) {
        config_if_entry_expr.if_cond.line += self.line_offset as u64;
        config_if_entry_expr.if_cond.end_line += self.line_offset as u64;

        for config_entry in config_if_entry_expr.items.iter_mut() {
            match config_entry.node.key.as_deref_mut() {
                Some(k) => {
                    k.line += self.line_offset as u64;
                    k.end_line += self.line_offset as u64;
                }
                None => (),
            }

            config_entry.node.value.line += self.line_offset as u64;
        }

        match config_if_entry_expr.orelse.as_deref_mut() {
            Some(v) => {
                v.line += self.line_offset as u64;
                v.end_line += self.line_offset as u64;
            }
            None => (),
        }

        self.walk_expr(&mut config_if_entry_expr.if_cond.node);
        for config_entry in config_if_entry_expr.items.iter_mut() {
            walk_if_mut!(self, walk_expr, config_entry.node.key);
            self.walk_expr(&mut config_entry.node.value.node);
        }
        walk_if_mut!(self, walk_expr, config_if_entry_expr.orelse);
    }
    fn walk_comp_clause(&mut self, comp_clause: &'ctx mut ast::CompClause) {
        comp_clause.iter.line += self.line_offset as u64;
        comp_clause.iter.end_line += self.line_offset as u64;

        comp_clause.targets.iter_mut().for_each(|t| {
            t.line += self.line_offset as u64;
            t.end_line += self.line_offset as u64;
        });

        comp_clause.ifs.iter_mut().for_each(|i| {
            i.line += self.line_offset as u64;
            i.end_line += self.line_offset as u64;
        });

        walk_list_mut!(self, walk_identifier, comp_clause.targets);
        self.walk_expr(&mut comp_clause.iter.node);
        walk_list_mut!(self, walk_expr, comp_clause.ifs);
    }
    fn walk_schema_expr(&mut self, schema_expr: &'ctx mut ast::SchemaExpr) {
        schema_expr.name.line += self.line_offset as u64;
        schema_expr.name.end_line += self.line_offset as u64;

        schema_expr.args.iter_mut().for_each(|a| {
            a.line += self.line_offset as u64;
            a.end_line += self.line_offset as u64;
        });

        schema_expr.kwargs.iter_mut().for_each(|k| {
            k.line += self.line_offset as u64;
            k.end_line += self.line_offset as u64;
        });

        schema_expr.config.line += self.line_offset as u64;
        schema_expr.config.end_line += self.line_offset as u64;

        self.walk_identifier(&mut schema_expr.name.node);
        walk_list_mut!(self, walk_expr, schema_expr.args);
        walk_list_mut!(self, walk_keyword, schema_expr.kwargs);
        self.walk_expr(&mut schema_expr.config.node);
    }
    fn walk_config_expr(&mut self, config_expr: &'ctx mut ast::ConfigExpr) {
        for config_entry in config_expr.items.iter_mut() {
            match config_entry.node.key.as_deref_mut() {
                Some(k) => {
                    k.line += self.line_offset as u64;
                    k.end_line += self.line_offset as u64;
                }
                None => (),
            }

            config_entry.node.value.line += self.line_offset as u64;
        }

        for config_entry in config_expr.items.iter_mut() {
            walk_if_mut!(self, walk_expr, config_entry.node.key);
            self.walk_expr(&mut config_entry.node.value.node);
        }
    }
    fn walk_check_expr(&mut self, check_expr: &'ctx mut ast::CheckExpr) {
        check_expr.test.line += self.line_offset as u64;
        check_expr.test.end_line += self.line_offset as u64;

        match check_expr.if_cond.as_deref_mut() {
            Some(v) => {
                v.line += self.line_offset as u64;
                v.end_line += self.line_offset as u64;
            }
            None => (),
        }

        match check_expr.msg.as_deref_mut() {
            Some(v) => {
                v.line += self.line_offset as u64;
                v.end_line += self.line_offset as u64;
            }
            None => (),
        }

        self.walk_expr(&mut check_expr.test.node);
        walk_if_mut!(self, walk_expr, check_expr.if_cond);
        walk_if_mut!(self, walk_expr, check_expr.msg);
    }
    fn walk_lambda_expr(&mut self, lambda_expr: &'ctx mut ast::LambdaExpr) {
        match lambda_expr.args.as_deref_mut() {
            Some(v) => {
                v.line += self.line_offset as u64;
                v.end_line += self.line_offset as u64;
            }
            None => (),
        }

        for stmt in lambda_expr.body.iter_mut() {
            stmt.line += self.line_offset as u64;
            stmt.end_line += self.line_offset as u64;
        }

        match lambda_expr.return_ty.as_deref_mut() {
            Some(v) => {
                v.line += self.line_offset as u64;
                v.end_line += self.line_offset as u64;
            }
            None => (),
        }

        walk_if_mut!(self, walk_arguments, lambda_expr.args);
        walk_list_mut!(self, walk_stmt, lambda_expr.body);
        walk_if_mut!(self, walk_type, lambda_expr.return_ty);
    }
    fn walk_keyword(&mut self, keyword: &'ctx mut ast::Keyword) {
        keyword.arg.line += self.line_offset as u64;
        keyword.arg.end_line += self.line_offset as u64;

        if let Some(v) = keyword.value.as_deref_mut() {
            v.line += self.line_offset as u64;
            v.end_line += self.line_offset as u64;
        }

        self.walk_identifier(&mut keyword.arg.node);
        if let Some(v) = keyword.value.as_deref_mut() {
            self.walk_expr(&mut v.node)
        }
    }
    fn walk_arguments(&mut self, arguments: &'ctx mut ast::Arguments) {
        arguments.args.iter_mut().for_each(|a| {
            a.line += self.line_offset as u64;
            a.end_line += self.line_offset as u64;
        });

        for default in arguments.defaults.iter_mut() {
            if let Some(d) = default.as_deref_mut() {
                d.line += self.line_offset as u64;
                d.end_line += self.line_offset as u64;
            }
        }
        for ty in arguments.ty_list.iter_mut() {
            if let Some(ty) = ty.as_deref_mut() {
                ty.line += self.line_offset as u64;
                ty.end_line += self.line_offset as u64;
            }
        }

        walk_list_mut!(self, walk_identifier, arguments.args);
        for default in arguments.defaults.iter_mut() {
            if let Some(d) = default.as_deref_mut() {
                self.walk_expr(&mut d.node)
            }
        }
        for ty in arguments.ty_list.iter_mut() {
            if let Some(ty) = ty.as_deref_mut() {
                self.walk_type(&mut ty.node);
            }
        }
    }
    fn walk_compare(&mut self, compare: &'ctx mut ast::Compare) {
        compare.left.line += self.line_offset as u64;
        compare.left.end_line += self.line_offset as u64;

        for comparator in compare.comparators.iter_mut() {
            comparator.line += self.line_offset as u64;
            comparator.end_line += self.line_offset as u64;
        }

        self.walk_expr(&mut compare.left.node);
        walk_list_mut!(self, walk_expr, compare.comparators);
    }
    fn walk_identifier(&mut self, identifier: &'ctx mut ast::Identifier) {
        // Nothing to do.
        let _ = identifier;
    }
    fn walk_number_lit(&mut self, number_lit: &'ctx mut ast::NumberLit) {
        let _ = number_lit;
    }
    fn walk_string_lit(&mut self, string_lit: &'ctx mut ast::StringLit) {
        // Nothing to do.
        let _ = string_lit;
    }
    fn walk_name_constant_lit(&mut self, name_constant_lit: &'ctx mut ast::NameConstantLit) {
        // Nothing to do.
        let _ = name_constant_lit;
    }
    fn walk_joined_string(&mut self, joined_string: &'ctx mut ast::JoinedString) {
        joined_string.values.iter_mut().for_each(|v| {
            v.line += self.line_offset as u64;
            v.end_line += self.line_offset as u64;
        });

        walk_list_mut!(self, walk_expr, joined_string.values);
    }
    fn walk_formatted_value(&mut self, formatted_value: &'ctx mut ast::FormattedValue) {
        formatted_value.value.line += self.line_offset as u64;
        formatted_value.value.end_line += self.line_offset as u64;

        self.walk_expr(&mut formatted_value.value.node);
    }
    fn walk_comment(&mut self, comment: &'ctx mut ast::Comment) {
        // Nothing to do.
        let _ = comment;
    }
    fn walk_missing_expr(&mut self, missing_expr: &'ctx mut ast::MissingExpr) {
        // Nothing to do.
        let _ = missing_expr;
    }
    fn walk_module(&mut self, module: &'ctx mut ast::Module) {
        module.comments.iter_mut().for_each(|c| {
            c.line += self.line_offset as u64;
            c.end_line += self.line_offset as u64;
        });

        for stmt in module.body.iter_mut() {
            if let ast::Stmt::Import(_) = stmt.node {
                continue;
            }

            stmt.line += self.line_offset as u64;
            stmt.end_line += self.line_offset as u64;

            self.walk_stmt(&mut stmt.node)
        }
    }
    fn walk_stmt(&mut self, stmt: &'ctx mut ast::Stmt) {
        match stmt {
            ast::Stmt::TypeAlias(type_alias) => self.walk_type_alias_stmt(type_alias),
            ast::Stmt::Expr(expr_stmt) => self.walk_expr_stmt(expr_stmt),
            ast::Stmt::Unification(unification_stmt) => {
                self.walk_unification_stmt(unification_stmt)
            }
            ast::Stmt::Assign(assign_stmt) => self.walk_assign_stmt(assign_stmt),
            ast::Stmt::AugAssign(aug_assign_stmt) => self.walk_aug_assign_stmt(aug_assign_stmt),
            ast::Stmt::Assert(assert_stmt) => self.walk_assert_stmt(assert_stmt),
            ast::Stmt::If(if_stmt) => self.walk_if_stmt(if_stmt),
            ast::Stmt::Import(import_stmt) => self.walk_import_stmt(import_stmt),
            ast::Stmt::SchemaAttr(schema_attr) => self.walk_schema_attr(schema_attr),
            ast::Stmt::Schema(schema_stmt) => self.walk_schema_stmt(schema_stmt),
            ast::Stmt::Rule(rule_stmt) => self.walk_rule_stmt(rule_stmt),
        }
    }
    fn walk_expr(&mut self, expr: &'ctx mut ast::Expr) {
        match expr {
            ast::Expr::Identifier(identifier) => self.walk_identifier(identifier),
            ast::Expr::Unary(unary_expr) => self.walk_unary_expr(unary_expr),
            ast::Expr::Binary(binary_expr) => self.walk_binary_expr(binary_expr),
            ast::Expr::If(if_expr) => self.walk_if_expr(if_expr),
            ast::Expr::Selector(selector_expr) => self.walk_selector_expr(selector_expr),
            ast::Expr::Call(call_expr) => self.walk_call_expr(call_expr),
            ast::Expr::Paren(paren_expr) => self.walk_paren_expr(paren_expr),
            ast::Expr::Quant(quant_expr) => self.walk_quant_expr(quant_expr),
            ast::Expr::List(list_expr) => self.walk_list_expr(list_expr),
            ast::Expr::ListIfItem(list_if_item_expr) => {
                self.walk_list_if_item_expr(list_if_item_expr)
            }
            ast::Expr::ListComp(list_comp) => self.walk_list_comp(list_comp),
            ast::Expr::Starred(starred_expr) => self.walk_starred_expr(starred_expr),
            ast::Expr::DictComp(dict_comp) => self.walk_dict_comp(dict_comp),
            ast::Expr::ConfigIfEntry(config_if_entry_expr) => {
                self.walk_config_if_entry_expr(config_if_entry_expr)
            }
            ast::Expr::CompClause(comp_clause) => self.walk_comp_clause(comp_clause),
            ast::Expr::Schema(schema_expr) => self.walk_schema_expr(schema_expr),
            ast::Expr::Config(config_expr) => self.walk_config_expr(config_expr),
            ast::Expr::Check(check) => self.walk_check_expr(check),
            ast::Expr::Lambda(lambda) => self.walk_lambda_expr(lambda),
            ast::Expr::Subscript(subscript) => self.walk_subscript(subscript),
            ast::Expr::Keyword(keyword) => self.walk_keyword(keyword),
            ast::Expr::Arguments(arguments) => self.walk_arguments(arguments),
            ast::Expr::Compare(compare) => self.walk_compare(compare),
            ast::Expr::NumberLit(number_lit) => self.walk_number_lit(number_lit),
            ast::Expr::StringLit(string_lit) => self.walk_string_lit(string_lit),
            ast::Expr::NameConstantLit(name_constant_lit) => {
                self.walk_name_constant_lit(name_constant_lit)
            }
            ast::Expr::JoinedString(joined_string) => self.walk_joined_string(joined_string),
            ast::Expr::FormattedValue(formatted_value) => {
                self.walk_formatted_value(formatted_value)
            }
            ast::Expr::Missing(missing_expr) => self.walk_missing_expr(missing_expr),
        }
    }
}
