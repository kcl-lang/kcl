use crate::ast;

/// Try get a config expr mut ref from a expr if the expr is a schema or a config.
/// If not, return [None].
/// TODO: use [TryInto]?
///
/// # Examples
///
/// ```
/// use kclvm_parser::parse_expr;
/// use kclvm_ast::ast;
/// use kclvm_ast::config::try_get_config_expr_mut;
///
/// let mut expr = parse_expr(r#"{
///     a: {b: {c = 1}}
/// }
/// "#).unwrap();
/// assert!(matches!(try_get_config_expr_mut(&mut expr.node), Some(_)));
/// let mut expr = parse_expr(r#"1"#).unwrap();
/// assert!(matches!(try_get_config_expr_mut(&mut expr.node), None));
/// ```
pub fn try_get_config_expr_mut(expr: &mut ast::Expr) -> Option<&mut ast::ConfigExpr> {
    match expr {
        ast::Expr::Schema(schema_expr) => {
            if let ast::Expr::Config(config_expr) = &mut schema_expr.config.node {
                Some(config_expr)
            } else {
                None
            }
        }
        ast::Expr::Config(config_expr) => Some(config_expr),
        _ => None,
    }
}
