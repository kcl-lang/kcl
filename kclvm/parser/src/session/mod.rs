use anyhow::Result;
use compiler_base_macros::bug;
use compiler_base_session::Session;
use indexmap::IndexSet;
use kclvm_ast::token::Token;
use kclvm_error::{Diagnostic, Handler, ParseError, ParseErrorMessage};
use kclvm_span::{BytePos, Loc, Span};
use parking_lot::RwLock;
use std::sync::Arc;

pub type ParseSessionRef = Arc<ParseSession>;

/// ParseSession represents the data associated with a parse session such as the
/// source map and the error handler.
#[derive(Default)]
pub struct ParseSession(pub Arc<Session>, pub RwLock<Handler>);

impl ParseSession {
    /// New a parse session with the global session.
    #[inline]
    pub fn with_session(sess: Arc<Session>) -> Self {
        Self(sess, RwLock::new(Handler::default()))
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

    /// Construct an loc of ont token.
    pub fn token_loc(&self, tok: Token) -> (Loc, Loc) {
        (
            self.lookup_char_pos(tok.span.lo()),
            self.lookup_char_pos(tok.span.hi()),
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
        self.add_parse_err(ParseError::String {
            message: msg.to_string(),
            span,
        });
    }

    #[inline]
    pub fn struct_message_error(&self, msg: ParseErrorMessage, span: Span) {
        self.add_parse_err(ParseError::Message {
            message: msg,
            span,
            suggestions: None,
        });
    }

    #[inline]
    pub fn struct_message_error_with_suggestions(
        &self,
        msg: ParseErrorMessage,
        span: Span,
        suggestions: Option<Vec<String>>,
    ) {
        self.add_parse_err(ParseError::Message {
            message: msg,
            span,
            suggestions,
        });
    }

    /// Add a error into the session.
    #[inline]
    fn add_parse_err(&self, err: ParseError) {
        let add_error = || -> Result<()> {
            self.0.add_err(err.clone().into_diag(&self.0)?)?;
            self.1.write().add_diagnostic(err.into_diag(&self.0)?);
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
            self.1.write().add_diagnostic(diagnostic);
        }
        self
    }

    /// Classify diagnostics into errors and warnings.
    pub fn classification(&self) -> (IndexSet<Diagnostic>, IndexSet<Diagnostic>) {
        self.1.read().classification()
    }
}
