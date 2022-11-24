//! KCL indent handling.
//! See details defined in KCL Grammar ['./spec/grammar'].

use std::cmp::Ordering;

use crate::lexer::IndentOrDedents;
use crate::lexer::Lexer;
use kclvm_ast::token::{self, Token};
use kclvm_error::Diagnostic;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub(crate) struct IndentLevel {
    pub(crate) tabs: usize,
    pub(crate) spaces: usize,
}

impl IndentLevel {
    pub(crate) fn cmp(&self, other: &IndentLevel) -> Result<Ordering, &'static str> {
        match self.tabs.cmp(&other.tabs) {
            Ordering::Less => {
                if self.spaces <= other.spaces {
                    Ok(Ordering::Less)
                } else {
                    Err("inconsistent use of tabs and spaces in indentation")
                }
            }
            Ordering::Greater => {
                if self.spaces >= other.spaces {
                    Ok(Ordering::Greater)
                } else {
                    Err("inconsistent use of tabs and spaces in indentation")
                }
            }
            Ordering::Equal => Ok(self.spaces.cmp(&other.spaces)),
        }
    }
}

impl<'a> Lexer<'a> {
    pub(crate) fn lex_indent_context(
        &mut self,
        token: kclvm_lexer::TokenKind,
    ) -> Result<Option<IndentOrDedents>, Diagnostic> {
        // process for indent context for a newline
        if !self.indent_cxt.new_line_beginning {
            return Ok(None);
        }

        match token {
            kclvm_lexer::TokenKind::LineComment { doc_style: _ }
            | kclvm_lexer::TokenKind::Newline => {
                // No in(de)ent in comment line and new line
                self.indent_cxt.tabs = 0;
                self.indent_cxt.spaces = 0;
                Ok(None)
            }
            kclvm_lexer::TokenKind::Tab => {
                self.indent_cxt.tabs += 1;
                Ok(None)
            }
            kclvm_lexer::TokenKind::Space => {
                self.indent_cxt.spaces += 1;
                Ok(None)
            }
            _ => {
                // End of detect on unrelated token, then do lex indent.
                self.indent_cxt.new_line_beginning = false;
                self.lex_indent()
            }
        }
    }

    fn lex_indent(&mut self) -> Result<Option<IndentOrDedents>, Diagnostic> {
        let tabs = self.indent_cxt.tabs;
        let spaces = self.indent_cxt.spaces;
        // reset counters
        self.indent_cxt.tabs = 0;
        self.indent_cxt.spaces = 0;

        // process indent at the end of the newline
        let mut cur_indent = self.indent_cxt.indents.last().unwrap();
        let indet = IndentLevel { tabs, spaces };
        let mut ordering = indet.cmp(cur_indent);

        match ordering {
            Ok(order) => {
                Ok(Some(match order {
                    Ordering::Greater => {
                        self.indent_cxt.indents.push(indet);

                        // For indent token, we ignore the length
                        let indent = Token::new(token::Indent, self.span(self.pos, self.pos));
                        IndentOrDedents::Indent { token: indent }
                    }
                    Ordering::Less => {
                        let mut dedents = Vec::new();

                        loop {
                            match ordering {
                                Ok(order) => {
                                    match order {
                                        Ordering::Less => {
                                            // Pop indents util we find an equal ident level
                                            self.indent_cxt.indents.pop();
                                            // update pos & collect dedent
                                            // For dedent token, we ignore the length
                                            let dedent = Token::new(
                                                token::Dedent,
                                                self.span(self.pos, self.pos),
                                            );
                                            dedents.push(dedent);
                                        }
                                        Ordering::Equal => {
                                            // Proper indent level found.
                                            break;
                                        }
                                        Ordering::Greater => {
                                            return Err(self.sess.struct_span_error(
                                                "fatal: logic error on dedenting.",
                                                self.span(self.pos, self.pos),
                                            ))
                                        }
                                    }

                                    // update cur indent and ordering
                                    cur_indent = self.indent_cxt.indents.last().unwrap();
                                    ordering = indet.cmp(cur_indent);
                                }
                                Err(msg) => {
                                    return Err(self
                                        .sess
                                        .struct_span_error(msg, self.span(self.pos, self.pos)))
                                }
                            }
                        }

                        IndentOrDedents::Dedents { tokens: dedents }
                    }
                    _ => return Ok(None),
                }))
            }
            Err(msg) => {
                return Err(self
                    .sess
                    .struct_span_error(msg, self.span(self.pos, self.pos)))
            }
        }
    }
}
