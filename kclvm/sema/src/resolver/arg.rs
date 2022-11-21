use crate::resolver::Resolver;
use crate::ty::{Parameter, Type};
use indexmap::IndexSet;
use kclvm_ast::ast;
use std::rc::Rc;

use crate::resolver::pos::GetPos;

impl<'ctx> Resolver<'ctx> {
    fn get_func_name(&mut self, func: &ast::Expr) -> String {
        let mut callee = func;
        loop {
            match callee {
                ast::Expr::Identifier(identifier) => {
                    return format!("\"{}\"", identifier.get_name());
                }
                ast::Expr::Selector(selector_expr) => {
                    return format!("\"{}\"", selector_expr.attr.node.get_name());
                }
                ast::Expr::Paren(paren_expr) => callee = &paren_expr.expr.node,
                _ => return "anonymous function".to_string(),
            }
        }
    }

    /// Do schema/function/decorator argument type check.
    pub fn do_arguments_type_check(
        &mut self,
        func: &ast::Expr,
        args: &'ctx [ast::NodeRef<ast::Expr>],
        kwargs: &'ctx [ast::NodeRef<ast::Keyword>],
        params: &[Parameter],
    ) {
        let func_name = self.get_func_name(func);
        let arg_types = self.exprs(args);
        let mut kwarg_types: Vec<(String, Rc<Type>)> = vec![];
        let mut check_table: IndexSet<String> = IndexSet::default();
        for kw in kwargs {
            let arg_name = &kw.node.arg.node.names[0];
            if check_table.contains(arg_name) {
                self.handler.add_compile_error(
                    &format!("{} has duplicated keyword argument {}", func_name, arg_name),
                    kw.get_pos(),
                );
            }
            check_table.insert(arg_name.to_string());
            let arg_value_type = self.expr_or_any_type(&kw.node.value);
            kwarg_types.push((arg_name.to_string(), arg_value_type.clone()));
        }
        if !params.is_empty() {
            for (i, ty) in arg_types.iter().enumerate() {
                let expected_ty = match params.get(i) {
                    Some(param) => param.ty.clone(),
                    None => {
                        self.handler.add_compile_error(
                            &format!(
                                "{} takes {} positional argument but {} were given",
                                func_name,
                                params.len(),
                                args.len(),
                            ),
                            args[i].get_pos(),
                        );
                        return;
                    }
                };
                self.must_assignable_to(ty.clone(), expected_ty, args[i].get_pos(), None)
            }
            for (i, (arg_name, kwarg_ty)) in kwarg_types.iter().enumerate() {
                if !params
                    .iter()
                    .map(|p| p.name.clone())
                    .any(|x| x == *arg_name)
                {
                    self.handler.add_compile_error(
                        &format!(
                            "{} got an unexpected keyword argument '{}'",
                            func_name, arg_name
                        ),
                        kwargs[i].get_pos(),
                    );
                }
                let expected_types: Vec<Rc<Type>> = params
                    .iter()
                    .filter(|p| p.name == *arg_name)
                    .map(|p| p.ty.clone())
                    .collect();
                if !expected_types.is_empty() {
                    self.must_assignable_to(
                        kwarg_ty.clone(),
                        expected_types[0].clone(),
                        kwargs[i].get_pos(),
                        None,
                    );
                };
            }
        }
    }
}
