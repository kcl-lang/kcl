use compiler_base_session::Session;
use kclvm_ast::token::Token;
use kclvm_error::{ParseError, Position};
use kclvm_runtime::PanicInfo;
use kclvm_span::{BytePos, Loc, Span};
use std::sync::Arc;

/// ParseSession represents the data associated with a parse session such as the
/// source map and the error handler.
pub struct ParseSession(pub Arc<Session>);

impl ParseSession {
    /// New a parse session with the global session.
    #[inline]
    pub fn with_session(sess: Arc<Session>) -> Self {
        Self(sess)
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

    /// Struct and report an error based on a token and abort the compiler process.
    pub fn struct_token_error(&self, expected: &[String], got: Token) -> ! {
        self.struct_token_error_recovery(expected, got);
        self.panic(
            &ParseError::UnexpectedToken {
                expected: expected.iter().map(|tok| tok.into()).collect(),
                got: got.into(),
                span: got.span,
            }
            .to_string(),
            got.span,
        );
    }

    /// Struct and report an error based on a token and not abort the compiler process.
    pub fn struct_token_error_recovery(&self, expected: &[String], got: Token) {
        let err = ParseError::UnexpectedToken {
            expected: expected.iter().map(|tok| tok.into()).collect(),
            got: got.into(),
            span: got.span,
        };
        self.0.add_err(err).unwrap();
    }

    /// Struct and report an error based on a span and abort the compiler process.
    pub fn struct_span_error(&self, msg: &str, span: Span) -> ! {
        self.struct_span_error_recovery(msg, span);
        self.panic(msg, span);
    }

    /// Struct and report an error based on a span and not abort the compiler process.
    #[inline]
    pub fn struct_span_error_recovery(&self, msg: &str, span: Span) {
        self.0
            .add_err(ParseError::Message {
                message: msg.to_string(),
                span,
            })
            .unwrap();
    }

    /// Parser panic with message and span.
    ///
    /// TODO: We can remove the panic capture after the parser error recovery is completed.
    fn panic(&self, msg: &str, span: Span) -> ! {
        let pos: Position = self.lookup_char_pos(span.lo()).into();
        let mut panic_info = PanicInfo::from(format!("Invalid syntax: {msg}"));
        panic_info.kcl_file = pos.filename.clone();
        panic_info.kcl_line = pos.line as i32;
        panic_info.kcl_col = pos.column.unwrap_or(0) as i32;
        panic!("{}", panic_info.to_json_string());
    }
}
