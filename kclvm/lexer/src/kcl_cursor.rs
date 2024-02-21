//! KCL-specific cursor implementation,
//! including comment cursor, string cursor and identifier cursor.
//!
//! Todo: The implementation should be moved to [`parser::lexer`].
//! To do that, we should make IABCCursor as dynamic traits
//! and enable implemente Cursor structs in different crate.

use crate::cursor::DOLLAR_CHAR;
use crate::cursor::EOF_CHAR;
use crate::Cursor;
use crate::DocStyle;
use crate::ICommentCursor;
use crate::IIdentCursor;
use crate::IStringCursor;
use crate::Literal;
use crate::LiteralKind::*;
use crate::TokenKind;
use crate::TokenKind::*;

impl<'a> ICommentCursor for Cursor<'a> {
    fn try_comment_magic(&self, c: char) -> bool {
        c == '#'
    }

    fn eat_comment(&mut self) -> TokenKind {
        debug_assert!(self.prev() == '#');

        self.eat_while(|c| c != '\n');
        LineComment {
            doc_style: Some(DocStyle::Inner),
        }
    }
}

impl<'a> IStringCursor for Cursor<'a> {
    fn try_string_magic(&self, c: char) -> bool {
        match c {
            'r' | 'R' => matches!(self.peek(), '\'' | '\"'),
            '\'' | '\"' => true,
            _ => false,
        }
    }

    fn eat_string(&mut self, c: char) -> TokenKind {
        match c {
            'r' | 'R' => match self.peek() {
                '\'' | '\"' => {
                    // R string
                    let quote = self.bump().unwrap_or(EOF_CHAR);
                    self.eat_quoted_string(quote)
                }
                _ => Unknown,
            },
            '\'' | '\"' => self.eat_quoted_string(c),
            _ => Unknown,
        }
    }
}

impl<'a> Cursor<'a> {
    // Eat (single | double | triple) quoted string.
    // If string is not closed, mark 'terminated' as false.
    // Note, it does not check whether the string content is correct in the quick scan here.
    // For example, it's not checking for newlines in single-line strings.
    fn eat_quoted_string(&mut self, c: char) -> TokenKind {
        debug_assert!(self.prev() == '\'' || self.prev() == '\"');

        let quote = c;

        // Check if we have a triple-quoted string, and make sure we have a triple-quote to close
        let triple_quoted = if quote == self.peek() && quote == self.peek1th() {
            self.bump();
            self.bump();
            true
        } else {
            false
        };

        while let Some(c) = self.bump() {
            match c {
                '\\' if self.peek() == '\\' || self.peek() == quote => {
                    // Skip the escaped quote
                    self.bump();
                }
                c if c == quote => {
                    if triple_quoted {
                        if quote == self.peek() && quote == self.peek1th() {
                            self.bump();
                            self.bump();

                            // Triple quote string closed
                            return Literal {
                                kind: Str {
                                    terminated: true,
                                    triple_quoted: true,
                                },
                                suffix_start: self.len_consumed(),
                            };
                        }
                    } else {
                        // Single or double quote string closed
                        return Literal {
                            kind: Str {
                                terminated: true,
                                triple_quoted: false,
                            },
                            suffix_start: self.len_consumed(),
                        };
                    }
                }
                // If we encounter an unclosed single quote string,
                // we end at the eof and newline characters '\r' or '\n'.
                _ if !triple_quoted && matches!(self.peek(), '\r' | '\n' | EOF_CHAR) => break,
                _ => (),
            }
        }

        // Oops, we get an error here, string not closed
        Literal {
            kind: Str {
                terminated: false,
                triple_quoted: false,
            },
            suffix_start: self.len_consumed(),
        }
    }
}

impl<'a> IIdentCursor for Cursor<'a> {
    fn try_ident_magic(&self, c: char) -> bool {
        match c {
            DOLLAR_CHAR => rustc_lexer::is_id_start(self.peek()),
            _ => rustc_lexer::is_id_start(c),
        }
    }

    fn eat_ident(&mut self) -> TokenKind {
        debug_assert!(rustc_lexer::is_id_start(self.prev()) || self.prev() == DOLLAR_CHAR);
        // Start is already eaten, eat the rest of identifier.
        self.eat_while(rustc_lexer::is_id_continue);
        // Known prefixes must have been handled earlier. So if
        // we see a prefix here, it is definitely an unknown prefix.
        match self.peek() {
            c if !c.is_ascii() && unic_emoji_char::is_emoji(c) => {
                self.fake_ident_or_unknown_prefix()
            }
            _ => Ident,
        }
    }
}
