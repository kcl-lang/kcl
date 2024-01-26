use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::builtin::BUILTIN_DECORATORS;
use crate::resolver::Resolver;
use crate::ty::{Decorator, DecoratorTarget, TypeKind};
use kclvm_ast::ast;
use kclvm_ast::pos::GetPos;
use kclvm_ast::walker::MutSelfTypedResultWalker;
use kclvm_ast_pretty::{print_ast_node, ASTNode};
use kclvm_error::{ErrorKind, Message, Position, Style};

use super::node::ResolvedResult;
use super::scope::{ScopeKind, ScopeObject, ScopeObjectKind};

impl<'ctx> Resolver<'ctx> {
    pub(crate) fn resolve_schema_stmt(
        &mut self,
        schema_stmt: &'ctx ast::SchemaStmt,
    ) -> ResolvedResult {
        let (start, end) = (self.ctx.start_pos.clone(), self.ctx.end_pos.clone());
        self.resolve_unique_key(&schema_stmt.name.node, &schema_stmt.name.get_span_pos());
        let ty =
            self.lookup_type_from_scope(&schema_stmt.name.node, schema_stmt.name.get_span_pos());
        self.node_ty_map
            .insert(self.get_node_key(schema_stmt.name.id.clone()), ty.clone());
        let scope_ty = if ty.is_schema() {
            ty.into_schema_type()
        } else {
            self.handler.add_error(
                ErrorKind::TypeError,
                &[Message {
                    range: schema_stmt.get_span_pos(),
                    style: Style::LineAndColumn,
                    message: format!("expected schema type, got {}", ty.ty_str()),
                    note: None,
                    suggested_replacement: None,
                }],
            );
            return ty;
        };
        self.ctx.schema = Some(Rc::new(RefCell::new(scope_ty.clone())));
        if let Some(args) = &schema_stmt.args {
            for (i, arg) in args.node.args.iter().enumerate() {
                let ty = args.node.get_arg_type_node(i);
                let ty = self.parse_ty_with_scope(ty, arg.get_span_pos());
                if let Some(name) = arg.node.names.last() {
                    self.node_ty_map
                        .insert(self.get_node_key(name.id.clone()), ty.clone());
                }
            }
        }
        self.do_parameters_check(&schema_stmt.args);
        self.enter_scope(
            start.clone(),
            end.clone(),
            ScopeKind::Schema(schema_stmt.name.node.to_string()),
        );
        for param in &scope_ty.func.params {
            self.insert_object(
                &param.name,
                ScopeObject {
                    name: param.name.clone(),
                    start: start.clone(),
                    end: end.clone(),
                    ty: param.ty.clone(),
                    kind: ScopeObjectKind::Parameter,
                    doc: None,
                },
            )
        }
        // Schema index signature
        if let (Some(index_signature), Some(index_signature_node)) =
            (scope_ty.index_signature, &schema_stmt.index_signature)
        {
            // Insert the schema index signature key name into the scope.
            if let Some(key_name) = index_signature.key_name {
                let (start, end) = index_signature_node.get_span_pos();
                self.insert_object(
                    &key_name,
                    ScopeObject {
                        name: key_name.clone(),
                        start,
                        end,
                        ty: index_signature.key_ty.clone(),
                        kind: ScopeObjectKind::Variable,
                        doc: None,
                    },
                )
            }
            // Check index signature default value type.
            if let Some(value) = &index_signature_node.node.value {
                let expected_ty = index_signature.val_ty;
                let value_ty = self.expr(value);
                self.must_assignable_to(
                    value_ty,
                    expected_ty,
                    index_signature_node.get_span_pos(),
                    None,
                );
            }
        }
        let schema_attr_names = schema_stmt.get_left_identifier_list();
        for (line, column, name) in schema_attr_names {
            if !self.contains_object(&name) {
                self.insert_object(
                    &name,
                    ScopeObject {
                        name: name.clone(),
                        start: Position {
                            filename: self.ctx.filename.clone(),
                            line,
                            column: Some(column),
                        },
                        end: Position::dummy_pos(),
                        ty: self.any_ty(),
                        kind: ScopeObjectKind::Variable,
                        doc: None,
                    },
                );
            }
        }
        // Schema body.
        self.stmts(&schema_stmt.body);
        // Schema check blocks.
        for check_expr in &schema_stmt.checks {
            self.walk_check_expr(&check_expr.node);
        }
        self.leave_scope();
        self.ctx.schema = None;
        ty
    }

    pub(crate) fn resolve_rule_stmt(&mut self, rule_stmt: &'ctx ast::RuleStmt) -> ResolvedResult {
        self.resolve_unique_key(&rule_stmt.name.node, &rule_stmt.name.get_span_pos());
        let ty = self.lookup_type_from_scope(&rule_stmt.name.node, rule_stmt.name.get_span_pos());
        self.node_ty_map
            .insert(self.get_node_key(rule_stmt.name.id.clone()), ty.clone());
        let scope_ty = if ty.is_schema() {
            ty.into_schema_type()
        } else {
            self.handler.add_error(
                ErrorKind::TypeError,
                &[Message {
                    range: rule_stmt.get_span_pos(),
                    style: Style::LineAndColumn,
                    message: format!("expected rule type, got {}", ty.ty_str()),
                    note: None,
                    suggested_replacement: None,
                }],
            );
            return ty;
        };
        self.ctx.schema = Some(Rc::new(RefCell::new(scope_ty.clone())));
        let (start, end) = rule_stmt.get_span_pos();
        self.do_parameters_check(&rule_stmt.args);
        self.enter_scope(
            start.clone(),
            end.clone(),
            ScopeKind::Schema(rule_stmt.name.node.to_string()),
        );
        for param in &scope_ty.func.params {
            self.insert_object(
                &param.name,
                ScopeObject {
                    name: param.name.clone(),
                    start: start.clone(),
                    end: end.clone(),
                    ty: param.ty.clone(),
                    kind: ScopeObjectKind::Parameter,
                    doc: None,
                },
            )
        }
        // Rule check blocks.
        for check_expr in &rule_stmt.checks {
            self.walk_check_expr(&check_expr.node);
        }
        self.leave_scope();
        self.ctx.schema = None;
        ty
    }

    pub(crate) fn resolve_decorators(
        &mut self,
        decorators: &'ctx [ast::NodeRef<ast::CallExpr>],
        target: DecoratorTarget,
        key: &str,
    ) -> Vec<Decorator> {
        let mut decorator_objs = vec![];
        for decorator in decorators {
            let name = if let ast::Expr::Identifier(identifier) = &decorator.node.func.node {
                if identifier.names.len() == 1 {
                    Some(identifier.names[0].node.clone())
                } else {
                    None
                }
            } else {
                None
            };
            match name {
                Some(name) => match BUILTIN_DECORATORS.get(&name) {
                    Some(ty) => match &ty.kind {
                        TypeKind::Function(func_ty) => {
                            self.do_arguments_type_check(
                                &decorator.node.func,
                                &decorator.node.args,
                                &decorator.node.keywords,
                                &func_ty,
                            );
                            let (arguments, keywords) = self.arguments_to_string(
                                &decorator.node.args,
                                &decorator.node.keywords,
                            );
                            decorator_objs.push(Decorator {
                                target: target.clone(),
                                name,
                                key: key.to_string(),
                                arguments,
                                keywords,
                            })
                        }
                        _ => bug!("invalid builtin decorator function type"),
                    },
                    None => {
                        self.handler.add_compile_error(
                            &format!("UnKnown decorator {}", name),
                            decorator.get_span_pos(),
                        );
                    }
                },
                None => {
                    self.handler.add_type_error(
                        "decorator name must be a single identifier",
                        decorator.get_span_pos(),
                    );
                }
            }
        }
        decorator_objs
    }

    fn arguments_to_string(
        &mut self,
        args: &'ctx [ast::NodeRef<ast::Expr>],
        kwargs: &'ctx [ast::NodeRef<ast::Keyword>],
    ) -> (Vec<String>, HashMap<String, String>) {
        if self.options.resolve_val {
            (
                args.iter()
                    .map(|a| print_ast_node(ASTNode::Expr(a)))
                    .collect(),
                kwargs
                    .iter()
                    .map(|a| {
                        (
                            a.node.arg.node.get_name(),
                            a.node
                                .value
                                .as_ref()
                                .map(|v| print_ast_node(ASTNode::Expr(v)))
                                .unwrap_or_default(),
                        )
                    })
                    .collect(),
            )
        } else {
            (vec![], HashMap::new())
        }
    }
}
