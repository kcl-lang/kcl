use kclvm_ast::walker::MutSelfMutWalker;
use kclvm_ast::{ast, walk_if_mut};
use kclvm_parser::parse_expr;

pub struct OverrideInfo {
    pub pkgpath: String,
    pub filename: String,
    pub module: ast::Module,
}

pub fn apply_overrides(
    prog: &mut ast::Program,
    overrides: &[ast::CmdOverrideSpec],
    _import_paths: &[String],
) {
    for o in overrides {
        let pkgpath = if o.pkgpath.is_empty() {
            &prog.main
        } else {
            &o.pkgpath
        };
        match prog.pkgs.get_mut(pkgpath) {
            Some(modules) => {
                for m in modules.iter_mut() {
                    if fix_module_override(m, o) {}
                    // module_add_import_paths(m, import_paths)
                }
            }
            None => {}
        }
    }
}

pub fn fix_module_override(m: &mut ast::Module, o: &ast::CmdOverrideSpec) -> bool {
    let ss = o.field_path.split(".").collect::<Vec<&str>>();
    if ss.len() <= 1 {
        false
    } else {
        let target_id = ss[0];
        let field = ss[1..].join(".");
        let value = &o.field_value;
        let key = ast::Identifier {
            names: field.split(".").map(|s| s.to_string()).collect(),
            ctx: ast::ExprContext::Store,
            pkgpath: "".to_string(),
        };
        let val = build_node_from_string(value);
        let mut transformer = OverrideTransformer {
            target_id: target_id.to_string(),
            field_path: field,
            override_key: key,
            override_value: val,
            override_target_count: 0,
            has_override: false,
            action: o.action.clone(),
        };
        transformer.walk_module(m);
        transformer.has_override
    }
}

pub fn build_node_from_string(value: &str) -> ast::NodeRef<ast::Expr> {
    let expr = parse_expr(value);
    expr
}

pub struct OverrideTransformer {
    pub target_id: String,
    pub field_path: String,
    pub override_key: ast::Identifier,
    pub override_value: ast::NodeRef<ast::Expr>,
    pub override_target_count: usize,
    pub has_override: bool,
    pub action: ast::OverrideAction,
}

impl<'ctx> MutSelfMutWalker<'ctx> for OverrideTransformer {
    fn walk_schema_stmt(&mut self, _: &'ctx mut ast::SchemaStmt) {
        // Do not override AssignStmt in SchemaStmt
    }

    fn walk_unification_stmt(&mut self, unification_stmt: &'ctx mut ast::UnificationStmt) {
        if unification_stmt.target.node.names[0] != self.target_id {
            return;
        }
        self.override_target_count = 1;
        self.has_override = true;
        self.walk_schema_expr(&mut unification_stmt.value.node);
    }

    fn walk_assign_stmt(&mut self, assign_stmt: &'ctx mut ast::AssignStmt) {
        if let ast::Expr::Schema(_) = &assign_stmt.value.node {
            self.override_target_count = 0;
            for target in &assign_stmt.targets {
                if target.node.names.len() != 1 {
                    continue;
                }
                if target.node.names[0] != self.target_id {
                    continue;
                }
                self.override_target_count += 1;
            }
            if self.override_target_count == 0 {
                return;
            }
            self.has_override = true;
            self.walk_expr(&mut assign_stmt.value.node);
        }
    }

    fn walk_schema_expr(&mut self, schema_expr: &'ctx mut ast::SchemaExpr) {
        if self.override_target_count == 0 {
            return;
        }
        if true {
            // Not exist and append an override value when the action is CREATE_OR_UPDATE
            if let ast::OverrideAction::CreateOrUpdate = self.action {
                if let ast::Expr::Config(config_expr) = &mut schema_expr.config.node {
                    config_expr
                        .items
                        .push(Box::new(ast::Node::dummy_node(ast::ConfigEntry {
                            key: Some(Box::new(ast::Node::dummy_node(ast::Expr::Identifier(
                                self.override_key.clone(),
                            )))),
                            value: self.override_value.clone(),
                            operation: ast::ConfigEntryOperation::Override,
                            insert_index: -1,
                        })));
                }
            }
        }
        self.override_target_count = 0;
    }

    fn walk_config_expr(&mut self, config_expr: &'ctx mut ast::ConfigExpr) {
        for config_entry in config_expr.items.iter_mut() {
            walk_if_mut!(self, walk_expr, config_entry.node.key);
            self.walk_expr(&mut config_entry.node.value.node);
        }
    }
}

impl OverrideTransformer {
    pub(crate) fn _get_schema_config_field_paths(
        &mut self,
        schema_expr: &mut ast::SchemaExpr,
    ) -> (Vec<String>, Vec<String>) {
        if let ast::Expr::Config(config_expr) = &mut schema_expr.config.node {
            self._get_config_field_paths(config_expr)
        } else {
            (vec![], vec![])
        }
    }
    pub(crate) fn _get_config_field_paths(
        &mut self,
        config: &mut ast::ConfigExpr,
    ) -> (Vec<String>, Vec<String>) {
        let mut paths = vec![];
        let mut paths_with_id = vec![];
        for entry in config.items.iter_mut() {
            let (mut _paths, mut _paths_with_id) = self._get_key_value_paths(&mut entry.node);
            paths.append(&mut _paths);
            paths_with_id.append(&mut &mut _paths_with_id);
        }
        (paths, paths_with_id)
    }
    pub(crate) fn _get_key_value_paths(
        &mut self,
        _entry: &mut ast::ConfigEntry,
    ) -> (Vec<String>, Vec<String>) {
        (vec![], vec![])
    }
    pub(crate) fn _find_schema_config_and_repalce(
        &mut self,
        _schema_config: &mut ast::SchemaExpr,
        _field_path: &str,
        _value: &ast::NodeRef<ast::Expr>,
    ) -> bool {
        false
    }
}
