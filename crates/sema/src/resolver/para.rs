use crate::resolver::Resolver;
use kcl_ast::ast;
use kcl_ast::pos::GetPos;
use kcl_error::*;

impl<'ctx> Resolver<'_> {
    /// Do parameter type check.
    pub fn do_parameters_check(&mut self, args: &'ctx Option<ast::NodeRef<ast::Arguments>>) {
        if let Some(args) = args {
            let mut mark = false;
            let len = args.node.defaults.len();
            for i in 0..len {
                let j = len - i - 1;
                match &args.node.defaults[j] {
                    Some(default) => {
                        if mark {
                            self.handler.add_error(
                                ErrorKind::IllegalParameterError,
                                &[Message {
                                    range: default.get_span_pos(),
                                    style: Style::LineAndColumn,
                                    message: "non-default argument follows default argument"
                                        .to_string(),
                                    note: Some("A default argument".to_string()),
                                    suggested_replacement: None,
                                }],
                            );
                        }
                    }
                    None => mark = true,
                }
            }
        }
    }
}
