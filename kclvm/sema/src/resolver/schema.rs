use std::cell::RefCell;
use std::rc::Rc;

use crate::builtin::BUILTIN_DECORATORS;
use crate::resolver::pos::GetPos;
use crate::resolver::Resolver;
use crate::ty::{Decorator, DecoratorTarget, TypeKind};
use kclvm_ast::ast;
use kclvm_ast::walker::MutSelfTypedResultWalker;
use kclvm_error::Position;

use super::node::ResolvedResult;
use super::scope::{ScopeKind, ScopeObject, ScopeObjectKind};

impl<'ctx> Resolver<'ctx> {
    pub(crate) fn resolve_schema_stmt(
        &mut self,
        schema_stmt: &'ctx ast::SchemaStmt,
    ) -> ResolvedResult {
        let ty = self.lookup_type_from_scope(&schema_stmt.name.node, schema_stmt.name.get_pos());
        let scope_ty = ty.into_schema_type();
        self.ctx.schema = Some(Rc::new(RefCell::new(scope_ty.clone())));
        let (start, end) = schema_stmt.get_span_pos();
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
                    used: false,
                },
            )
        }
        // Schema index signature
        if let (Some(index_signature), Some(index_signature_node)) =
            (scope_ty.index_signature, &schema_stmt.index_signature)
        {
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
                        used: false,
                    },
                )
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
                        used: false,
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
        let ty = self.lookup_type_from_scope(&rule_stmt.name.node, rule_stmt.name.get_pos());
        let scope_ty = ty.into_schema_type();
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
                    used: false,
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
                    Some(identifier.names[0].clone())
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
                                &decorator.node.args,
                                &decorator.node.keywords,
                                &func_ty.params,
                            );
                            decorator_objs.push(Decorator {
                                target: target.clone(),
                                name,
                                key: key.to_string(),
                            })
                        }
                        _ => bug!("invalid builtin decorator function type"),
                    },
                    None => {
                        self.handler.add_compile_error(
                            &format!("UnKnown decorator {}", name),
                            decorator.get_pos(),
                        );
                    }
                },
                None => {
                    self.handler.add_type_error(
                        "decorator name must be a single identifier",
                        decorator.get_pos(),
                    );
                }
            }
        }
        decorator_objs
    }
}
