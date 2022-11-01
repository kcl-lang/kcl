use kclvm::{ErrType, PanicInfo};
use kclvm_ast::token::Token;
use kclvm_error::{Handler, ParseError, Position};
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

    /// Struct and report an error based on a token and abort the compiler process.
    pub fn struct_token_error(&self, expected: &[String], got: Token) -> ! {
        let pos: Position = self.source_map.lookup_char_pos(got.span.lo()).into();
        let err = ParseError::UnexpectedToken {
            expected: expected.iter().map(|tok| tok.into()).collect(),
            got: got.into(),
        };

        let mut panic_info = PanicInfo::default();

        panic_info.__kcl_PanicInfo__ = true;
        panic_info.message = format!("{:?}", err);
        panic_info.err_type_code = ErrType::CompileError_TYPE as i32;

        panic_info.kcl_file = pos.filename.clone();
        panic_info.kcl_line = pos.line as i32;
        panic_info.kcl_col = pos.column.unwrap_or(0) as i32;

        panic!("{}", panic_info.to_json_string())
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
    pub fn struct_span_error(&self, msg: &str, span: Span) -> ! {
        let pos: Position = self.source_map.lookup_char_pos(span.lo()).into();

        let mut panic_info = PanicInfo::default();

        panic_info.__kcl_PanicInfo__ = true;
        panic_info.message = format!("Invalid syntax: {}", msg);
        panic_info.err_type_code = ErrType::CompileError_TYPE as i32;

        panic_info.kcl_file = pos.filename.clone();
        panic_info.kcl_line = pos.line as i32;
        panic_info.kcl_col = pos.column.unwrap_or(0) as i32;

        panic!("{}", panic_info.to_json_string())
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
