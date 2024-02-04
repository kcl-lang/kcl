use std::collections::HashSet;

use anyhow::{anyhow, Result};

use compiler_base_macros::bug;
use kclvm_ast::config::try_get_config_expr_mut;
use kclvm_ast::path::get_key_path;
use kclvm_ast::walker::MutSelfMutWalker;
use kclvm_ast::MAIN_PKG;
use kclvm_ast::{ast, walk_if_mut};
use kclvm_ast_pretty::print_ast_module;
use kclvm_parser::parse_expr;
use kclvm_sema::pre_process::{fix_config_expr_nest_attr, transform_multi_assign};

use crate::path::parse_attribute_path;

use super::util::{invalid_spec_error, split_field_path};

/// Import statement column offset always start with 1.
/// todo: The (1-based) column offset needs to be constrained by specifications.
const IMPORT_STMT_COLUMN_OFFSET: u64 = 1;

/// Apply overrides on the AST program with the override specifications.
///
/// Please note that this a low level internal API used by compiler itself,
/// The parameters of the method are all compiler internal concepts such as
/// AST, etc.
///
/// # Examples
///
/// ```no_check
/// use kclvm_parser::load_program;
/// use kclvm_tools::query::r#override::apply_overrides;
///
/// let mut prog = load_program(&["config.k"], None, None).unwrap();
/// let overrides = vec![parse_override_spec("config.id=1").unwrap()];
/// let import_paths = vec!["path.to.pkg".to_string()];
/// let result = apply_overrides(&mut prog, &overrides, &import_paths, true).unwrap();
/// ```
pub fn apply_overrides(
    prog: &mut ast::Program,
    overrides: &[ast::OverrideSpec],
    import_paths: &[String],
    print_ast: bool,
) -> Result<()> {
    for o in overrides {
        let pkgpath = if o.pkgpath.is_empty() {
            MAIN_PKG
        } else {
            &o.pkgpath
        };
        if let Some(modules) = prog.pkgs.get_mut(pkgpath) {
            for m in modules.iter_mut() {
                if apply_override_on_module(m, o, import_paths)? && print_ast {
                    let code_str = print_ast_module(m);
                    std::fs::write(&m.filename, &code_str)?
                }
            }
        }
    }
    Ok(())
}

/// Build a expression from string.
fn build_expr_from_string(value: &str) -> Option<ast::NodeRef<ast::Expr>> {
    let expr: Option<ast::NodeRef<ast::Expr>> = parse_expr(value);
    match &expr {
        Some(e) => match &e.node {
            // fix attr=value to attr="value"
            ast::Expr::Identifier(_) | ast::Expr::Unary(_) | ast::Expr::Binary(_) => {
                Some(ast::NodeRef::new(ast::Node::node_with_pos(
                    ast::Expr::StringLit(ast::StringLit {
                        is_long_string: false,
                        raw_value: format!("{value:?}"),
                        value: value.to_string(),
                    }),
                    e.pos(),
                )))
            }
            _ => expr,
        },
        None => None,
    }
}

/// Apply overrides on the AST module with the override specifications.
///
/// Please note that this a low level internal API used by compiler itself,
/// The parameters of the method are all compiler internal concepts such as
/// AST, etc.
///
/// # Examples
///
/// ```no_check
/// use kclvm_parser::parse_file_force_errors;
/// use kclvm_tools::query::apply_override_on_module;
///
/// let mut module = parse_file_force_errors("", None).unwrap();
/// let override_spec = parse_override_spec("config.id=1").unwrap();
/// let import_paths = vec!["path.to.pkg".to_string()];
/// let result = apply_override_on_module(&mut module, override_spec, &import_paths).unwrap();
/// ```
pub fn apply_override_on_module(
    m: &mut ast::Module,
    o: &ast::OverrideSpec,
    import_paths: &[String],
) -> Result<bool> {
    // Apply import paths on AST module.
    apply_import_paths_on_module(m, import_paths)?;
    let ss = parse_attribute_path(&o.field_path)?;
    if ss.len() <= 1 {
        return Ok(false);
    }
    let target_id = &ss[0];
    let value = &o.field_value;
    let key = ast::Identifier {
        names: ss[1..]
            .iter()
            .map(|s| ast::Node::dummy_node(s.to_string()))
            .collect(),
        ctx: ast::ExprContext::Store,
        pkgpath: "".to_string(),
    };
    // Transform config expr to simplify the config path query and override.
    fix_config_expr_nest_attr(m);
    // When there is a multi-target assignment statement of the form `a = b = Config {}`,
    // it needs to be transformed into the following form first to prevent the configuration
    // from being incorrectly modified.
    // ```kcl
    // a = Config {}
    // b = Config {}
    // ```
    transform_multi_assign(m);
    let mut transformer = OverrideTransformer {
        target_id: target_id.to_string(),
        field_paths: ss[1..].to_vec(),
        override_key: key,
        override_value: build_expr_from_string(value),
        override_target_count: 0,
        has_override: false,
        action: o.action.clone(),
    };
    transformer.walk_module(m);
    Ok(transformer.has_override)
}

/// Parse override spec string to override structure.
///
/// parse_override_spec("alice.age=10") -> ast::OverrideSpec {
///     pkgpath: "".to_string(),
///     field_path: "alice.age".to_string(),
///     field_value: "10".to_string(),
///     action: ast::OverrideAction::CreateOrUpdate,
/// }
pub fn parse_override_spec(spec: &str) -> Result<ast::OverrideSpec> {
    if spec.contains('=') {
        // Create or update the override value.
        let split_values = spec.splitn(2, '=').collect::<Vec<&str>>();
        let path = split_values
            .first()
            .ok_or_else(|| invalid_spec_error(spec))?;
        let field_value = split_values
            .get(1)
            .ok_or_else(|| invalid_spec_error(spec))?;
        let (pkgpath, field_path) = split_field_path(path)?;
        Ok(ast::OverrideSpec {
            pkgpath,
            field_path,
            field_value: field_value.to_string(),
            action: ast::OverrideAction::CreateOrUpdate,
        })
    } else if let Some(stripped_spec) = spec.strip_suffix('-') {
        // Delete the override value.
        let (pkgpath, field_path) = split_field_path(stripped_spec)?;
        Ok(ast::OverrideSpec {
            pkgpath,
            field_path,
            field_value: "".to_string(),
            action: ast::OverrideAction::Delete,
        })
    } else {
        Err(invalid_spec_error(spec))
    }
}

// Transform the AST module with the import path list.
fn apply_import_paths_on_module(m: &mut ast::Module, import_paths: &[String]) -> Result<()> {
    if import_paths.is_empty() {
        return Ok(());
    }
    let mut exist_import_set: HashSet<String> = HashSet::new();
    for stmt in &m.body {
        if let ast::Stmt::Import(import_stmt) = &stmt.node {
            if let Some(asname) = &import_stmt.asname {
                exist_import_set.insert(format!("{} as {}", import_stmt.path.node, asname.node));
            } else {
                exist_import_set.insert(import_stmt.path.node.to_string());
            }
        }
    }
    for (i, path) in import_paths.iter().enumerate() {
        let line: u64 = i as u64 + 1;
        if exist_import_set.contains(path) {
            continue;
        }
        let name = path
            .split('.')
            .last()
            .ok_or_else(|| anyhow!("Invalid import path {}", path))?;
        let import_node = ast::ImportStmt {
            path: ast::Node::dummy_node(path.to_string()),
            rawpath: "".to_string(),
            name: name.to_string(),
            asname: None,
            pkg_name: String::new(),
        };
        let import_stmt = Box::new(ast::Node::new(
            ast::Stmt::Import(import_node),
            m.filename.clone(),
            line,
            IMPORT_STMT_COLUMN_OFFSET,
            line,
            // i denotes the space len between the `import` keyword and the path.
            ("import".len() + path.len() + 1) as u64,
        ));
        m.body.insert((line - 1) as usize, import_stmt)
    }
    Ok(())
}

/// OverrideTransformer is used to walk AST and transform it with the override values.
struct OverrideTransformer {
    pub target_id: String,
    pub field_paths: Vec<String>,
    pub override_key: ast::Identifier,
    pub override_value: Option<ast::NodeRef<ast::Expr>>,
    pub override_target_count: usize,
    pub has_override: bool,
    pub action: ast::OverrideAction,
}

impl<'ctx> MutSelfMutWalker<'ctx> for OverrideTransformer {
    fn walk_unification_stmt(&mut self, unification_stmt: &'ctx mut ast::UnificationStmt) {
        let name = match unification_stmt.target.node.names.get(0) {
            Some(name) => name,
            None => bug!(
                "Invalid AST unification target names {:?}",
                unification_stmt.target.node.names
            ),
        };
        if name.node != self.target_id {
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
                if target.node.names[0].node != self.target_id {
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
        if let ast::Expr::Config(config_expr) = &mut schema_expr.config.node {
            if !self.lookup_config_and_replace(config_expr) {
                // Not exist and append an override value when the action is CREATE_OR_UPDATE
                if let ast::OverrideAction::CreateOrUpdate = self.action {
                    if let ast::Expr::Config(config_expr) = &mut schema_expr.config.node {
                        config_expr
                            .items
                            .push(Box::new(ast::Node::dummy_node(ast::ConfigEntry {
                                key: Some(Box::new(ast::Node::dummy_node(ast::Expr::Identifier(
                                    self.override_key.clone(),
                                )))),
                                value: self.clone_override_value(),
                                operation: ast::ConfigEntryOperation::Override,
                                insert_index: -1,
                            })));
                    }
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

    fn walk_if_stmt(&mut self, _: &'ctx mut ast::IfStmt) {
        // Do not override AssignStmt in IfStmt
    }
    fn walk_schema_stmt(&mut self, _: &'ctx mut ast::SchemaStmt) {
        // Do not override AssignStmt in SchemaStmt
    }
    fn walk_lambda_expr(&mut self, _: &'ctx mut ast::LambdaExpr) {
        // Do not override AssignStmt in LambdaExpr
    }
}

impl OverrideTransformer {
    /// Lookup schema config all fields and replace if it is matched with the override spec,
    /// return whether is found a replaced one.
    fn lookup_config_and_replace(&self, config_expr: &mut ast::ConfigExpr) -> bool {
        // Split a path into multiple parts. `a.b.c` -> ["a", "b", "c"]
        let parts = self
            .field_paths
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();
        self.replace_config_with_path_parts(config_expr, &parts)
    }

    /// Replace AST config expr with one part of path. The implementation of this function
    /// uses recursive matching to find the config entry need to be modified.
    fn replace_config_with_path_parts(
        &self,
        config_expr: &mut ast::ConfigExpr,
        parts: &[&str],
    ) -> bool {
        // Do not replace empty path parts and out of index parts on the config expression.
        if parts.is_empty() {
            return false;
        }
        // Always take the first part to match, because recursive search is required.
        let part = parts[0];
        let mut delete_index_set = HashSet::new();
        let mut changed = false;
        // Loop all entries in the config expression and replace, because there may be duplicate
        // configuration items in config.
        for (i, item) in config_expr.items.iter_mut().enumerate() {
            // Compare each field of the config structure one by one.
            // - `part` denotes the path entered by the user to be modified.
            // - `get_path_key` returns the real config key name.
            // For example, the real config node is `a: {b: c: {}}`. The path
            // that needs to be modified is `a.b.c`, and its parts are ["a", "b", "c"].
            if part == get_key_path(&item.node.key) {
                // When the last part of the path is successfully recursively matched,
                // it indicates that the original value that needs to be overwritten
                // is successfully found, and the new value is used to overwrite it.
                // - `parts.len() == 1` denotes the path matches exactly.
                if parts.len() == 1 {
                    match self.action {
                        ast::OverrideAction::CreateOrUpdate => {
                            let mut value = self.clone_override_value();
                            // Use position information that needs to override the expression.
                            value.set_pos(item.pos());
                            // Override the node value.
                            item.node.value = value;
                            changed = true;
                        }
                        ast::OverrideAction::Delete => {
                            // Store the config entry delete index into the delete index set.
                            // Because we can't delete the entry directly in the loop
                            delete_index_set.insert(i);
                            changed = true;
                        }
                    }
                }
                // Replace value recursively using the path composed by subsequent parts.
                //
                // The reason for using recursion instead of looping for path matching
                // is that rust cannot directly hold shared references to AST nodes
                // (ast::NodeRef<T> is a Box<T>), so recursive search is performed
                // directly on AST nodes.
                else if let Some(config_expr) = try_get_config_expr_mut(&mut item.node.value.node)
                {
                    changed = self.replace_config_with_path_parts(config_expr, &parts[1..]);
                }
            }
        }
        // Delete entries according delete index set.
        if !delete_index_set.is_empty() {
            let items: Vec<(usize, &ast::NodeRef<ast::ConfigEntry>)> = config_expr
                .items
                .iter()
                .enumerate()
                .filter(|(i, _)| !delete_index_set.contains(i))
                .collect();
            config_expr.items = items
                .iter()
                .map(|(_, item)| <&ast::NodeRef<ast::ConfigEntry>>::clone(item).clone())
                .collect();
        } else if let ast::OverrideAction::CreateOrUpdate = self.action {
            if !changed {
                let key = ast::Identifier {
                    names: parts
                        .iter()
                        .map(|s| ast::Node::dummy_node(s.to_string()))
                        .collect(),
                    ctx: ast::ExprContext::Store,
                    pkgpath: "".to_string(),
                };
                config_expr
                    .items
                    .push(Box::new(ast::Node::dummy_node(ast::ConfigEntry {
                        key: Some(Box::new(ast::Node::dummy_node(ast::Expr::Identifier(key)))),
                        value: self.clone_override_value(),
                        operation: ast::ConfigEntryOperation::Override,
                        insert_index: -1,
                    })));
                changed = true;
            }
        }
        return changed;
    }

    /// Clone a override value
    #[inline]
    fn clone_override_value(&self) -> ast::NodeRef<ast::Expr> {
        match &self.override_value {
            Some(v) => v.clone(),
            None => bug!("Override value is None"),
        }
    }
}
