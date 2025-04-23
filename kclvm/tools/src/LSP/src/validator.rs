use kclvm_ast::ast::{Program, Node, Stmt, AssignStmt, Expr};
use kclvm_error::{Diagnostic, Level};
use kclvm_error::diagnostic::Position;
use kclvm_sema::resolver::scope::ProgramScope;
use std::collections::HashMap;

pub fn validate_schema_attributes(program: &Program, _scope: &ProgramScope) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();

    // Process schemas and validate instances in a single pass
    for (_, modules) in &program.pkgs {
        for module_path in modules {
            if let Ok(Some(module)) = program.get_module(module_path) {
                let mut schema_attrs = HashMap::new();
                
                for stmt in &module.body {
                    match &**stmt {
                        Node { node: Stmt::Schema(schema), .. } => {
                            let mut required_attrs = Vec::new();
                            for attr in &schema.body {
                                if let Node { node: Stmt::SchemaAttr(attr_stmt), .. } = &**attr {
                                    if !attr_stmt.is_optional && attr_stmt.value.is_none() {
                                        required_attrs.push(attr_stmt.name.node.clone());
                                    }
                                }
                            }
                            schema_attrs.insert(schema.name.node.clone(), required_attrs);
                        },
                        Node { node: Stmt::Assign(assign_stmt), filename, line, column, .. } => {
                            if let Some(schema_name) = get_schema_name(assign_stmt) {
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
                                                    filename: filename.clone(),
                                                    line: *line,
                                                    column: Some(*column),
                                                },
                                                Position {
                                                    filename: filename.clone(),
                                                    line: *line,
                                                    column: Some(*column + schema_name.len() as u64),
                                                }
                                            ),
                                        ));
                                    }
                                }
                            }
                        },
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

fn get_schema_name(assign_stmt: &AssignStmt) -> Option<String> {
    if let Node { node: Expr::Schema(schema_expr), .. } = &*assign_stmt.value {
        schema_expr.name.node.names.last().map(|n| n.node.clone())
    } else {
        None
    }
}

fn get_missing_attrs(assign_stmt: &AssignStmt, required_attrs: &[String]) -> Vec<String> {
    if let Node { node: Expr::Schema(schema_expr), .. } = &*assign_stmt.value {
        if let Node { node: Expr::Config(config_expr), .. } = &*schema_expr.config {
            let provided_attrs: Vec<String> = config_expr
                .items
                .iter()
                .filter_map(|item| {
                    item.node.key.as_ref().and_then(|key| {
                        if let Node { node: Expr::Identifier(ident), .. } = &**key {
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