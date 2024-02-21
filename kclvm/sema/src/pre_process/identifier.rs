use crate::info::is_private_field;
use indexmap::{IndexMap, IndexSet};
use kclvm_ast::ast::Node;
use kclvm_ast::pos::GetPos;
use kclvm_ast::walker::MutSelfMutWalker;
use kclvm_ast::{ast, walk_if_mut, walk_list_mut};
use kclvm_error::*;

pub const RAW_IDENTIFIER_PREFIX: &str = "$";

#[derive(Default)]
struct QualifiedIdentifierTransformer {
    pub import_names: IndexMap<String, String>,
    pub global_names: IndexMap<String, Position>,
    pub local_vars: IndexSet<String>,
    pub scope_level: usize,
}

impl<'ctx> MutSelfMutWalker<'ctx> for QualifiedIdentifierTransformer {
    fn walk_rule_stmt(&mut self, rule_stmt: &'ctx mut ast::RuleStmt) {
        let name = &rule_stmt.name.node;
        if !self.global_names.contains_key(name) && self.scope_level == 0 {
            self.global_names
                .insert(name.to_string(), rule_stmt.name.get_pos());
        }

        walk_list_mut!(self, walk_identifier, rule_stmt.parent_rules);
        walk_list_mut!(self, walk_call_expr, rule_stmt.decorators);
        walk_if_mut!(self, walk_arguments, rule_stmt.args);
        walk_if_mut!(self, walk_identifier, rule_stmt.for_host_name);
        self.scope_level += 1;
        walk_list_mut!(self, walk_check_expr, rule_stmt.checks);
        self.scope_level -= 1;
    }
    fn walk_schema_stmt(&mut self, schema_stmt: &'ctx mut ast::SchemaStmt) {
        let name = &schema_stmt.name.node;
        if !self.global_names.contains_key(name) && self.scope_level == 0 {
            self.global_names
                .insert(name.to_string(), schema_stmt.name.get_pos());
        }
        walk_if_mut!(self, walk_identifier, schema_stmt.parent_name);
        walk_if_mut!(self, walk_identifier, schema_stmt.for_host_name);
        walk_if_mut!(self, walk_arguments, schema_stmt.args);
        self.scope_level += 1;
        if let Some(schema_index_signature) = schema_stmt.index_signature.as_deref_mut() {
            let value = &mut schema_index_signature.node.value;
            walk_if_mut!(self, walk_expr, value);
        }
        walk_list_mut!(self, walk_identifier, schema_stmt.mixins);
        walk_list_mut!(self, walk_call_expr, schema_stmt.decorators);
        walk_list_mut!(self, walk_stmt, schema_stmt.body);
        walk_list_mut!(self, walk_check_expr, schema_stmt.checks);
        self.scope_level -= 1;
    }
    fn walk_assign_stmt(&mut self, assign_stmt: &'ctx mut ast::AssignStmt) {
        let is_config = matches!(assign_stmt.value.node, ast::Expr::Schema(_));
        for target in &assign_stmt.targets {
            if !target.node.names.is_empty() {
                let name = &target.node.names[0].node;
                if (is_private_field(name) || !self.global_names.contains_key(name) || is_config)
                    && self.scope_level == 0
                {
                    self.global_names.insert(name.to_string(), target.get_pos());
                }
            }
        }
        self.walk_expr(&mut assign_stmt.value.node);
    }
    fn walk_aug_assign_stmt(&mut self, aug_assign_stmt: &'ctx mut ast::AugAssignStmt) {
        let is_config = matches!(aug_assign_stmt.value.node, ast::Expr::Schema(_));
        if aug_assign_stmt.target.node.names.is_empty() {
            return;
        }
        let name = &aug_assign_stmt.target.node.names[0].node;
        if is_private_field(name) || !self.global_names.contains_key(name) || is_config {
            if self.scope_level == 0 {
                self.global_names
                    .insert(name.to_string(), aug_assign_stmt.target.get_pos());
            }
        }
        self.walk_expr(&mut aug_assign_stmt.value.node);
    }
    fn walk_schema_expr(&mut self, schema_expr: &'ctx mut ast::SchemaExpr) {
        self.walk_identifier(&mut schema_expr.name.node);
        walk_list_mut!(self, walk_expr, schema_expr.args);
        walk_list_mut!(self, walk_keyword, schema_expr.kwargs);
        self.walk_expr(&mut schema_expr.config.node);
    }
    fn walk_import_stmt(&mut self, _: &'ctx mut ast::ImportStmt) {}
    fn walk_lambda_expr(&mut self, lambda_expr: &'ctx mut ast::LambdaExpr) {
        walk_if_mut!(self, walk_arguments, lambda_expr.args);
        self.scope_level += 1;
        walk_list_mut!(self, walk_stmt, lambda_expr.body);
        self.scope_level -= 1;
    }
    fn walk_list_comp(&mut self, list_comp: &'ctx mut ast::ListComp) {
        for gen in &mut list_comp.generators {
            for target in &gen.node.targets {
                if !target.node.names.is_empty() {
                    self.local_vars
                        .insert(target.node.names[0].node.to_string());
                }
            }
        }
        self.walk_expr(&mut list_comp.elt.node);
        walk_list_mut!(self, walk_comp_clause, list_comp.generators);
        self.local_vars.clear();
    }
    fn walk_dict_comp(&mut self, dict_comp: &'ctx mut ast::DictComp) {
        for gen in &dict_comp.generators {
            for target in &gen.node.targets {
                if !target.node.names.is_empty() {
                    self.local_vars
                        .insert(target.node.names[0].node.to_string());
                }
            }
        }
        if let Some(key) = dict_comp.entry.key.as_deref_mut() {
            self.walk_expr(&mut key.node);
        }
        self.walk_expr(&mut dict_comp.entry.value.node);
        walk_list_mut!(self, walk_comp_clause, dict_comp.generators);
        self.local_vars.clear();
    }
    fn walk_quant_expr(&mut self, quant_expr: &'ctx mut ast::QuantExpr) {
        for target in &quant_expr.variables {
            if !target.node.names.is_empty() {
                self.local_vars
                    .insert(target.node.names[0].node.to_string());
            }
        }
        self.walk_expr(&mut quant_expr.target.node);
        self.walk_expr(&mut quant_expr.test.node);
        walk_if_mut!(self, walk_expr, quant_expr.if_cond);
        self.local_vars.clear();
    }
    fn walk_identifier(&mut self, identifier: &'ctx mut ast::Identifier) {
        if identifier.names.len() >= 2 {
            // skip global name and generator local variables in list/dict comp and quant expression
            let name = &identifier.names[0].node;
            if !self.global_names.contains_key(name) && !self.local_vars.contains(name) {
                if let Some(pkgpath) = self.import_names.get(name) {
                    identifier.pkgpath = pkgpath.clone()
                }
            }
        }
    }
}

#[inline]
fn remove_raw_ident_prefix(name: &str) -> String {
    match name.strip_prefix(RAW_IDENTIFIER_PREFIX) {
        Some(name_without_prefix) => name_without_prefix.to_string(),
        None => name.to_string(),
    }
}

#[derive(Debug, Default)]
struct RawIdentifierTransformer;

impl<'ctx> MutSelfMutWalker<'ctx> for RawIdentifierTransformer {
    fn walk_identifier(&mut self, identifier: &'ctx mut ast::Identifier) {
        identifier.names = identifier
            .names
            .iter()
            .map(|name| {
                Node::node_with_pos_and_id(
                    remove_raw_ident_prefix(&name.node),
                    name.pos(),
                    name.id.clone(),
                )
            })
            .collect::<Vec<Node<String>>>();
    }
    fn walk_schema_attr(&mut self, schema_attr: &'ctx mut ast::SchemaAttr) {
        // If the attribute is an identifier and then fix it.
        // Note that we do not fix a string-like attribute e.g., `"$name"`
        if schema_attr.name.end_column - schema_attr.name.column
            <= schema_attr.name.node.chars().count() as u64
        {
            schema_attr.name.node = remove_raw_ident_prefix(&schema_attr.name.node);
        }
        walk_list_mut!(self, walk_call_expr, schema_attr.decorators);
        walk_if_mut!(self, walk_expr, schema_attr.value);
    }
    fn walk_schema_stmt(&mut self, schema_stmt: &'ctx mut ast::SchemaStmt) {
        schema_stmt.name.node = remove_raw_ident_prefix(&schema_stmt.name.node);
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
        rule_stmt.name.node = remove_raw_ident_prefix(&rule_stmt.name.node);
        walk_list_mut!(self, walk_identifier, rule_stmt.parent_rules);
        walk_list_mut!(self, walk_call_expr, rule_stmt.decorators);
        walk_list_mut!(self, walk_check_expr, rule_stmt.checks);
        walk_if_mut!(self, walk_arguments, rule_stmt.args);
        walk_if_mut!(self, walk_identifier, rule_stmt.for_host_name);
    }
    fn walk_import_stmt(&mut self, import_stmt: &'ctx mut ast::ImportStmt) {
        if let Some(name) = &mut import_stmt.asname {
            name.node = remove_raw_ident_prefix(&name.node);
        }
        import_stmt.name = remove_raw_ident_prefix(&import_stmt.name);
        import_stmt.path.node = remove_raw_ident_prefix(&import_stmt.path.node);
    }
}

/// import path.to.pkg as pkgname
///
/// x = pkgname.Name
pub fn fix_qualified_identifier<'ctx>(
    module: &'ctx mut ast::Module,
    import_names: &mut IndexMap<String, String>,
) {
    // 0. init import names.
    for stmt in &module.body {
        if let ast::Stmt::Import(import_stmt) = &stmt.node {
            import_names.insert(import_stmt.name.clone(), import_stmt.path.node.clone());
        }
    }
    // 1. fix_global_ident
    let mut global_names_walker = QualifiedIdentifierTransformer {
        import_names: import_names.clone(),
        ..Default::default()
    };
    global_names_walker.walk_module(module);
}

/// Fix AST raw identifier prefix `$`, e.g., $filter -> filter
pub fn fix_raw_identifier_prefix(module: &'_ mut ast::Module) {
    RawIdentifierTransformer::default().walk_module(module);
}
