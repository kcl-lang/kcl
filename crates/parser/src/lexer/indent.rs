//! KCL indent handling.

use std::cmp::Ordering;

use crate::lexer::IndentOrDedents;
use crate::lexer::Lexer;
use kclvm_ast::token::VALID_SPACES_LENGTH;
use kclvm_ast::token::{self, Token};

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
    ) -> Option<IndentOrDedents> {
        // process for indent context for a newline
        if !self.indent_cxt.new_line_beginning {
            return None;
        }

        match token {
            kclvm_lexer::TokenKind::LineComment { doc_style: _ }
            | kclvm_lexer::TokenKind::Newline => {
                // No in(de)ent in comment line and new line
                self.indent_cxt.tabs = 0;
                self.indent_cxt.spaces = 0;
                None
            }
            kclvm_lexer::TokenKind::Tab => {
                self.indent_cxt.tabs += 1;
                None
            }
            kclvm_lexer::TokenKind::Space => {
                self.indent_cxt.spaces += 1;
                None
            }
            _ => {
                // End of detect on unrelated token, then do lex indent.
                self.indent_cxt.new_line_beginning = false;
                self.lex_indent()
            }
        }
    }

    fn lex_indent(&mut self) -> Option<IndentOrDedents> {
        let tabs = self.indent_cxt.tabs;
        let spaces = self.indent_cxt.spaces;
        // reset counters
        self.indent_cxt.tabs = 0;
        self.indent_cxt.spaces = 0;

        // process indent at the end of the newline
        let mut cur_indent = self.last_indent();
        let indent = IndentLevel { tabs, spaces };
        let mut ordering = indent.cmp(cur_indent);

        match ordering {
            Ok(order) => {
                Some(match order {
                    Ordering::Greater => {
                        self.indent_cxt.indents.push(indent);

                        // For indent token, we ignore the length
                        let indent = Token::new(
                            token::Indent(VALID_SPACES_LENGTH),
                            self.span(self.pos, self.pos),
                        );
                        IndentOrDedents::Indent { token: indent }
                    }
                    Ordering::Less => {
                        let mut dedents = Vec::new();
                        let mut indents = Vec::new();

                        loop {
                            match ordering {
                                Ok(order) => {
                                    match order {
                                        Ordering::Less => {
                                            // Pop indents util we find an equal ident level
                                            if let Some(indent) = self.indent_cxt.indents.pop() {
                                                indents.push(indent);
                                            }
                                            // update pos & collect dedent
                                            // For dedent token, we ignore the length
                                            let dedent = Token::new(
                                                token::Dedent(VALID_SPACES_LENGTH),
                                                self.span(self.pos, self.pos),
                                            );
                                            dedents.push(dedent);
                                        }
                                        Ordering::Equal => {
                                            // Proper indent level found.
                                            break;
                                        }
                                        Ordering::Greater => {
                                            let spaces_diff = indent.spaces - cur_indent.spaces;
                                            if let Some(indent) = indents.pop() {
                                                self.indent_cxt.indents.push(indent);
                                            }
                                            dedents.pop();
                                            dedents.push(Token::new(
                                                token::Dedent(spaces_diff),
                                                self.span(self.pos, self.pos),
                                            ));
                                            break;
                                        }
                                    }

                                    // update cur indent and ordering
                                    cur_indent = self.last_indent();
                                    ordering = indent.cmp(cur_indent);
                                }
                                Err(msg) => {
                                    self.sess
                                        .struct_span_error(msg, self.span(self.pos, self.pos));
                                    break;
                                }
                            }
                        }
                        IndentOrDedents::Dedents { tokens: dedents }
                    }
                    _ => return None,
                })
            }
            Err(msg) => {
                self.sess
                    .struct_span_error(msg, self.span(self.pos, self.pos));
                None
            }
        }
    }

    /// Get the last indent, if not exists, return a default level for error recovery.
    fn last_indent(&mut self) -> &IndentLevel {
        if self.indent_cxt.indents.is_empty() {
            self.sess
                .struct_span_error("mismatched indent level", self.span(self.pos, self.pos));
            self.indent_cxt.indents.push(IndentLevel::default());
        }
        self.indent_cxt.indents.last().unwrap()
    }
}
