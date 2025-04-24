use kclvm_ast::ast::{AssignStmt, Expr, Node, Program, Stmt};
use kclvm_error::diagnostic::Position;
use kclvm_error::{Diagnostic, Level};
use kclvm_sema::resolver::scope::ProgramScope;
use std::collections::HashMap;

pub fn validate_schema_attributes(
    program: &Program,
    _scope: &ProgramScope,
) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();

    // Process schemas and validate instances in a single pass
    for (_, modules) in &program.pkgs {
        for module_path in modules {
            if let Ok(Some(module)) = program.get_module(module_path) {
                let mut schema_attrs = HashMap::new();

                // First pass: collect all schema definitions
                for stmt in &module.body {
                    if let Node {
                        node: Stmt::Schema(schema),
                        ..
                    } = &**stmt
                    {
                        let mut required_attrs = Vec::new();
                        for attr in &schema.body {
                            if let Node {
                                node: Stmt::SchemaAttr(attr_stmt),
                                ..
                            } = &**attr
                            {
                                if !attr_stmt.is_optional && attr_stmt.value.is_none() {
                                    required_attrs.push(attr_stmt.name.node.clone());
                                }
                            }
                        }
                        schema_attrs.insert(schema.name.node.clone(), required_attrs);
                    }
                }

                // Second pass: validate all instances including nested ones and lambdas
                for stmt in &module.body {
                    match &**stmt {
                        Node {
                            node: Stmt::Assign(assign_stmt),
                            filename,
                            line,
                            column,
                            ..
                        } => {
                            validate_schema_instance(
                                assign_stmt,
                                &schema_attrs,
                                filename,
                                *line,
                                *column,
                                &mut diagnostics,
                            );

                            // Check if the assignment is a lambda that returns a schema
                            if let Node {
                                node: Expr::Lambda(lambda_expr),
                                ..
                            } = &*assign_stmt.value
                            {
                                if let Some(schema_expr) =
                                    get_schema_from_lambda_body(&lambda_expr.body)
                                {
                                    let nested_assign = AssignStmt {
                                        value: Box::new(Node {
                                            node: Expr::Schema(schema_expr.clone()),
                                            filename: filename.clone(),
                                            line: *line,
                                            column: *column,
                                            end_line: *line,
                                            end_column: *column,
                                            id: kclvm_ast::ast::AstIndex::default(),
                                        }),
                                        ..assign_stmt.clone()
                                    };
                                    validate_schema_instance(
                                        &nested_assign,
                                        &schema_attrs,
                                        filename,
                                        *line,
                                        *column,
                                        &mut diagnostics,
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn get_schema_from_lambda_body(body: &[Box<Node<Stmt>>]) -> Option<&kclvm_ast::ast::SchemaExpr> {
    for stmt in body {
        if let Node {
            node: Stmt::Expr(expr_stmt),
            ..
        } = &**stmt
        {
            if let Node {
                node: Expr::Schema(schema_expr),
                ..
            } = &*expr_stmt.exprs[0]
            {
                return Some(schema_expr);
            }
        }
    }
    None
}

fn validate_schema_instance(
    assign_stmt: &AssignStmt,
    schema_attrs: &HashMap<String, Vec<String>>,
    filename: &str,
    line: u64,
    column: u64,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Node {
        node: Expr::Schema(schema_expr),
        ..
    } = &*assign_stmt.value
    {
        let schema_name = schema_expr.name.node.names.last().unwrap().node.clone();

        if let Some(required_attrs) = schema_attrs.get(&schema_name) {
            let missing_attrs = get_missing_attrs(assign_stmt, required_attrs);
            if !missing_attrs.is_empty() {
                diagnostics.push(Diagnostic::new(
                    Level::Error,
                    &format!(
                        "Missing required attributes in {} instance: {}",
                        schema_name,
                        missing_attrs.join(", ")
                    ),
                    (
                        Position {
                            filename: filename.to_string(),
                            line,
                            column: Some(column),
                        },
                        Position {
                            filename: filename.to_string(),
                            line,
                            column: Some(column + schema_name.len() as u64),
                        },
                    ),
                ));
            }

            // Recursively validate nested schema instances
            if let Node {
                node: Expr::Config(config_expr),
                ..
            } = &*schema_expr.config
            {
                for item in &config_expr.items {
                    if let Node {
                        node: Expr::Schema(_),
                        ..
                    } = &*item.node.value
                    {
                        let nested_assign = AssignStmt {
                            value: item.node.value.clone(),
                            ..assign_stmt.clone()
                        };
                        validate_schema_instance(
                            &nested_assign,
                            schema_attrs,
                            filename,
                            line,
                            column,
                            diagnostics,
                        );
                    }
                }
            }
        }
    }
}

fn get_schema_name(assign_stmt: &AssignStmt) -> Option<String> {
    if let Node {
        node: Expr::Schema(schema_expr),
        ..
    } = &*assign_stmt.value
    {
        schema_expr.name.node.names.last().map(|n| n.node.clone())
    } else {
        None
    }
}

fn get_missing_attrs(assign_stmt: &AssignStmt, required_attrs: &[String]) -> Vec<String> {
    if let Node {
        node: Expr::Schema(schema_expr),
        ..
    } = &*assign_stmt.value
    {
        if let Node {
            node: Expr::Config(config_expr),
            ..
        } = &*schema_expr.config
        {
            let provided_attrs: Vec<String> = config_expr
                .items
                .iter()
                .filter_map(|item| {
                    item.node.key.as_ref().and_then(|key| {
                        if let Node {
                            node: Expr::Identifier(ident),
                            ..
                        } = &**key
                        {
                            ident.names.last().map(|n| n.node.clone())
                        } else {
                            None
                        }
                    })
                })
                .collect();

            required_attrs
                .iter()
                .filter(|attr| !provided_attrs.contains(attr))
                .cloned()
                .collect()
        } else {
            required_attrs.to_vec()
        }
    } else {
        Vec::new()
    }
}
