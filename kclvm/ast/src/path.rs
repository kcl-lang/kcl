use crate::ast;

/// Get config key path from the AST key node and convert string-based AST nodes including
/// `ast::Expr::Identifier` and `ast::Expr::StringLit` to strings.
///
/// # Examples
///
/// ```
/// use kclvm_ast::ast;
/// use kclvm_ast::path::get_key_path;
///
/// let ident = Some(Box::new(ast::Node::dummy_node(ast::Expr::Identifier(ast::Identifier {
///     names: vec![ast::Node::dummy_node("alice".to_string())],
///     pkgpath: "".to_string(),
///     ctx: ast::ExprContext::Load,
/// }))));
/// assert_eq!(get_key_path(&ident), "alice");
/// let str_lit = Some(Box::new(ast::Node::dummy_node(ast::Expr::StringLit(ast::StringLit {
///     is_long_string: false,
///     raw_value: "\"Alice\"".to_string(),
///     value: "Alice".to_string(),
/// }))));
/// assert_eq!(get_key_path(&str_lit), "Alice");
/// ```
#[inline]
pub fn get_key_path(key: &Option<ast::NodeRef<ast::Expr>>) -> String {
    match key {
        Some(key) => match &key.node {
            ast::Expr::Identifier(identifier) => identifier.get_name(),
            ast::Expr::StringLit(string_lit) => string_lit.value.clone(),
            _ => "".to_string(),
        },
        None => "".to_string(),
    }
}

/// Get all attribute paths recursively from a config expression AST node.
///
/// # Examples
///
/// ```
/// use kclvm_parser::parse_expr;
/// use kclvm_ast::ast;
/// use kclvm_ast::path::get_attr_paths_from_config_expr;
///
/// let expr = parse_expr(r#"{
///     a: {b: {c = 1}}
/// }
/// "#).unwrap();
/// if let ast::Expr::Config(config_expr) = &expr.node {
///     assert_eq!(get_attr_paths_from_config_expr(&config_expr), vec![
///         "a".to_string(),
///         "a.b".to_string(),
///         "a.b.c".to_string(),
///     ])
/// } else {
///     panic!("invalid config expr {:?}", expr)
/// }
/// ```
pub fn get_attr_paths_from_config_expr(config: &ast::ConfigExpr) -> Vec<String> {
    let mut paths = vec![];
    for entry in &config.items {
        let mut entry_paths = get_entry_paths(&entry.node);
        paths.append(&mut entry_paths);
    }
    paths
}

/// Get all attribute paths from a config entry.
fn get_entry_paths(entry: &ast::ConfigEntry) -> Vec<String> {
    let mut paths = vec![];
    let path = get_key_path(&entry.key);
    if path.is_empty() || path.trim().is_empty() {
        return paths;
    }
    paths.push(path.clone());
    let option_config_expr = match &entry.value.node {
        ast::Expr::Schema(schema_expr) => {
            if let ast::Expr::Config(config_expr) = &schema_expr.config.node {
                Some(config_expr)
            } else {
                None
            }
        }
        ast::Expr::Config(config_expr) => Some(config_expr),
        _ => None,
    };
    if let Some(config_expr) = option_config_expr {
        let value_paths = get_attr_paths_from_config_expr(config_expr);
        if !value_paths.is_empty() {
            paths.append(
                &mut value_paths
                    .iter()
                    .map(|value_path| format!("{}.{}", path, value_path))
                    .collect::<Vec<String>>(),
            );
        }
    }
    paths
}
