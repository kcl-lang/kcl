use kclvm_ast::token::Token;
use kclvm_error::{Diagnostic, Handler, ParseError, Position, E2L23};
use kclvm_span::{Loc, SourceMap, Span};
use std::cell::RefCell;
use std::sync::Arc;

pub struct ParseSession {
    pub source_map: Arc<SourceMap>,
    pub handler: RefCell<Handler>,
}

impl ParseSession {
    pub fn with_source_map(source_map: Arc<SourceMap>) -> Self {
        let handler = Handler::with_source_map(source_map.clone()).into();
        Self {
            handler,
            source_map,
        }
    }

    // Struct an loc of first and last valid tokens in an expr, returns a loc tuple
    pub fn struct_token_loc(&self, lot: Token, hit: Token) -> (Loc, Loc) {
        (
            self.source_map.lookup_char_pos(lot.span.lo()),
            self.source_map.lookup_char_pos(hit.span.hi()),
        )
    }

    /// Struct and report an error and abort the compiler process.
    pub fn struct_compile_error(&self, msg: &str, pos: Position) -> Diagnostic {
        self.handler
            .borrow_mut()
            .contract_diagnostic(E2L23.kind, msg, pos)
    }

    /// Struct and report an error based on a token and abort the compiler process.
    pub fn struct_token_error(&self, expected: &[String], got: Token) -> Diagnostic {
        let pos: Position = self.source_map.lookup_char_pos(got.span.lo()).into();
        let err = ParseError::UnexpectedToken {
            expected: expected.iter().map(|tok| tok.into()).collect(),
            got: got.into(),
        };
        self.handler
            .borrow_mut()
            .contract_diagnostic(E2L23.kind, &format!("{}", err), pos)
    }

    /// Struct and report an error based on a token and not abort the compiler process.
    pub fn struct_token_error_recovery(&self, expected: &[String], got: Token) {
        let pos: Position = self.source_map.lookup_char_pos(got.span.lo()).into();
        let err = ParseError::UnexpectedToken {
            expected: expected.iter().map(|tok| tok.into()).collect(),
            got: got.into(),
        };

        self.handler.borrow_mut().add_parse_error(err, pos);
    }

    /// Struct and report an error based on a span and abort the compiler process.
    pub fn struct_span_error(&self, msg: &str, span: Span) -> Diagnostic {
        let pos: Position = self.source_map.lookup_char_pos(span.lo()).into();
        self.struct_compile_error(&format!("{}", msg), pos)
    }

    /// Struct and report an error based on a span and not abort the compiler process.
    pub fn struct_span_error_recovery(&self, msg: &str, span: Span) {
        let pos: Position = self.source_map.lookup_char_pos(span.lo()).into();

        self.handler.borrow_mut().add_compile_error(msg, pos);
    }

    /// Report a compiler bug
    pub fn struct_compiler_bug(&self, msg: &str) -> ! {
        self.handler.borrow_mut().bug(msg)
    }
}
