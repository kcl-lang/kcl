use anyhow::Result;
use compiler_base_macros::bug;
use compiler_base_session::Session;
use indexmap::IndexSet;
use kclvm_ast::token::Token;
use kclvm_error::{Diagnostic, FixInfo, Handler, ParseError};
use kclvm_span::{BytePos, Loc, Span};
use std::{cell::RefCell, sync::Arc};

pub type ParseSessionRef = Arc<ParseSession>;

/// ParseSession represents the data associated with a parse session such as the
/// source map and the error handler.
#[derive(Default, Clone)]
pub struct ParseSession(pub Arc<Session>, pub RefCell<Handler>);

impl ParseSession {
    /// New a parse session with the global session.
    #[inline]
    pub fn with_session(sess: Arc<Session>) -> Self {
        Self(sess, RefCell::new(Handler::default()))
    }

    /// Lookup char pos from span.
    #[inline]
    pub(crate) fn lookup_char_pos(&self, pos: BytePos) -> Loc {
        self.0.sm.lookup_char_pos(pos)
    }

    /// Returns the source snippet as [String] corresponding to the given [Span].
    #[inline]
    pub fn span_to_snippet(&self, span: Span) -> String {
        self.0.sm.span_to_snippet(span).unwrap()
    }

    /// Struct an loc of first and last valid tokens in an expr, returns a loc tuple
    pub fn struct_token_loc(&self, lot: Token, hit: Token) -> (Loc, Loc) {
        (
            self.lookup_char_pos(lot.span.lo()),
            self.lookup_char_pos(hit.span.hi()),
        )
    }

    /// Struct and report an error based on a token and not abort the compiler process.
    #[inline]
    pub fn struct_token_error(&self, expected: &[String], got: Token) {
        self.add_parse_err(ParseError::UnexpectedToken {
            expected: expected.iter().map(|tok| tok.into()).collect(),
            got: got.into(),
            span: got.span,
        });
    }

    /// Struct and report an error based on a span and not abort the compiler process.
    #[inline]
    pub fn struct_span_error(&self, msg: &str, span: Span) {
        self.add_parse_err(ParseError::Message {
            message: msg.to_string(),
            span,
            fix_info: None,
        });
    }

    #[inline]
    pub fn struct_span_error_with_suggestions(
        &self,
        msg: &str,
        span: Span,
        suggestion_text: Option<String>,
        replacement_text: Option<String>,
    ) {
        let code_line = self.get_line_content_from_span(span);
        let modified_code_line =
            self.generate_modified_code_line(msg, &code_line, replacement_text.as_deref());

        let fix_info = Some(FixInfo {
            suggestion: suggestion_text,
            replacement: Some(modified_code_line),
        });

        self.add_parse_err(ParseError::Message {
            message: msg.to_string(),
            span,
            fix_info,
        });
    }

    fn generate_modified_code_line(
        &self,
        msg: &str,
        code_line: &str,
        replacement_text: Option<&str>,
    ) -> String {
        match msg {
            "invalid token '!', consider using 'not'" => {
                if let Some(replacement) = replacement_text {
                    code_line.replace("!", replacement)
                } else {
                    code_line.replace("!", "not ")
                }
            }
            "'else if' here is invalid in KCL, consider using the 'elif' keyword" => {
                if let Some(replacement) = replacement_text {
                    code_line.replace("else if", replacement)
                } else {
                    code_line.replace("else if", "elif")
                }
            }
            "error nesting on close paren"
            | "mismatched closing delimiter"
            | "error nesting on close brace" => {
                let mismatched_delim = self.find_mismatched_delimiter(code_line);
                if let Some(replacement) = replacement_text {
                    code_line.replace(&mismatched_delim, replacement)
                } else {
                    code_line.to_string()
                }
            }
            "unterminated string" => {
                if let Some(replacement) = replacement_text {
                    format!("{}{}", code_line, replacement)
                } else {
                    format!("{}\"", code_line)
                }
            }
            "unexpected character after line continuation character" => {
                if let Some(replacement) = replacement_text {
                    if let Some(pos) = code_line.find('\\') {
                        format!("{}{}", &code_line[..=pos - 1], replacement)
                    } else {
                        code_line.to_string()
                    }
                } else {
                    code_line.to_string()
                }
            }
            "the semicolon ';' here is unnecessary, please remove it" => code_line.replace(";", ""),
            _ => code_line.to_string(),
        }
    }

    fn find_mismatched_delimiter(&self, code_line: &str) -> String {
        if code_line.contains(")") {
            ")".to_string()
        } else if code_line.contains("]") {
            "]".to_string()
        } else if code_line.contains("}") {
            "}".to_string()
        } else {
            "".to_string()
        }
    }

    /// Extracts the content of the line corresponding to the given span.
    fn get_line_content_from_span(&self, span: Span) -> String {
        let source_file = self.0.sm.lookup_source_file(span.lo());
        let line_index = source_file.lookup_line(span.lo()).unwrap();
        source_file
            .get_line(line_index)
            .unwrap_or_else(|| "unknown line".into())
            .to_string()
    }

    /// Add a error into the session.
    #[inline]
    fn add_parse_err(&self, err: ParseError) {
        let add_error = || -> Result<()> {
            self.0.add_err(err.clone().into_diag(&self.0)?)?;
            self.1.borrow_mut().add_diagnostic(err.into_diag(&self.0)?);
            Ok(())
        };
        if let Err(err) = add_error() {
            bug!(
                "compiler session internal error occurs: {}",
                err.to_string()
            )
        }
    }

    /// Append diagnostics into the parse session.
    pub fn append_diagnostic(&self, diagnostics: IndexSet<Diagnostic>) -> &Self {
        for diagnostic in diagnostics {
            self.1.borrow_mut().add_diagnostic(diagnostic);
        }
        self
    }

    /// Classify diagnostics into errors and warnings.
    pub fn classification(&self) -> (IndexSet<Diagnostic>, IndexSet<Diagnostic>) {
        self.1.borrow().classification()
    }
}
