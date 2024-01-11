use super::*;
use crate::lexer::str_content_eval;
use crate::session::ParseSession;
use compiler_base_error::diagnostic_handler::DiagnosticHandler;
use compiler_base_session::Session;
use compiler_base_span::{span::new_byte_pos, FilePathMapping, SourceMap};
use expect_test::{expect, Expect};
use kclvm_error::Handler;
use kclvm_span::create_session_globals_then;
use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::Arc;

impl ParseSession {
    #[inline]
    pub(crate) fn with_source_map(sm: Arc<SourceMap>) -> Self {
        Self(
            Arc::new(Session::new(sm, Arc::new(DiagnosticHandler::default()))),
            RefCell::new(Handler::default()),
        )
    }
}

/// lexing the 'src'.
fn lex(src: &str) -> (String, String) {
    let sm = SourceMap::new(FilePathMapping::empty());
    sm.new_source_file(PathBuf::from("").into(), src.to_string());
    let sess = &ParseSession::with_source_map(Arc::new(sm));

    // preprocess the input str by [`SourceFile`]
    let sf = sess
        .0
        .sm
        .new_source_file(PathBuf::from("").into(), src.to_string());

    let src_from_sf = match sf.src.as_ref() {
        Some(src) => src,
        None => {
            panic!("Unreachable code")
        }
    };

    let res = create_session_globals_then(|| {
        parse_token_streams(sess, src_from_sf, new_byte_pos(0))
            .iter()
            .map(|token| format!("{:?}\n", token))
            .collect()
    });

    let err_msgs = sess
        .0
        .emit_all_diags_into_string()
        .unwrap()
        .iter()
        .map(|err| err.as_ref().unwrap().to_string())
        .collect();

    (res, err_msgs)
}

/// check the invalid panic message.
fn check_lexing_with_err_msg(src: &str, expect: Expect, expect_err_msg: Expect) {
    let (got, got_err) = lex(src);
    expect.assert_eq(&got);
    expect_err_msg.assert_eq(&got_err);
}

fn check_lexing(src: &str, expect: Expect) {
    expect.assert_eq(&lex(src).0);
}

// Get the code snippets from 'src' by token.span, and compare with expect.
fn check_span(src: &str, expect: Expect) {
    let sm = SourceMap::new(FilePathMapping::empty());
    sm.new_source_file(PathBuf::from("").into(), src.to_string());
    let sess = &ParseSession::with_source_map(Arc::new(SourceMap::new(FilePathMapping::empty())));

    create_session_globals_then(move || {
        let actual: String = parse_token_streams(sess, src, new_byte_pos(0))
            .iter()
            .map(|token| format!("{:?}\n", sm.span_to_snippet(token.span).unwrap()))
            .collect();
        expect.assert_eq(&actual)
    });
}

#[test]
fn test_str_content_eval() {
    let cases = [
        // true cases
        (("1", '\'', false, false, false), Some("1".to_string())),
        (("1", '"', false, false, false), Some("1".to_string())),
        (
            ("1\\n2", '"', false, false, false),
            Some("1\n2".to_string()),
        ),
        (
            ("1\\n2", '"', false, false, true),
            Some("1\\n2".to_string()),
        ),
        (("1\\2", '"', false, false, true), Some("1\\2".to_string())),
        (("1", '\'', true, false, false), Some("1".to_string())),
        (("1", '"', true, false, false), Some("1".to_string())),
        (("1\n2", '"', true, false, false), Some("1\n2".to_string())),
    ];
    for ((input, quote_char, triple_quoted, is_bytes, is_raw), expected) in cases {
        assert_eq!(
            str_content_eval(input, quote_char, triple_quoted, is_bytes, is_raw),
            expected,
            "test failed, input: {input}"
        )
    }
}

#[test]
fn smoke_test() {
    check_lexing(
        "lambda { println(\"kclvm\") }\n",
        expect![[r#"
        Token { kind: Ident(Symbol(SymbolIndex { idx: 18 })), span: Span { base_or_index: 0, len_or_tag: 6 } }
        Token { kind: OpenDelim(Brace), span: Span { base_or_index: 7, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 42 })), span: Span { base_or_index: 9, len_or_tag: 7 } }
        Token { kind: OpenDelim(Paren), span: Span { base_or_index: 16, len_or_tag: 1 } }
        Token { kind: Literal(Lit { kind: Str { is_long_string: false, is_raw: false }, symbol: Symbol(SymbolIndex { idx: 43 }), suffix: None, raw: Some(Symbol(SymbolIndex { idx: 44 })) }), span: Span { base_or_index: 17, len_or_tag: 7 } }
        Token { kind: CloseDelim(Paren), span: Span { base_or_index: 24, len_or_tag: 1 } }
        Token { kind: CloseDelim(Brace), span: Span { base_or_index: 26, len_or_tag: 1 } }
        Token { kind: Newline, span: Span { base_or_index: 27, len_or_tag: 1 } }
        Token { kind: Eof, span: Span { base_or_index: 28, len_or_tag: 0 } }
        "#]],
    )
}

#[test]
fn comment_flavors() {
    check_lexing(
        r"
# line
",
        expect![[r#"
        Token { kind: Newline, span: Span { base_or_index: 0, len_or_tag: 1 } }
        Token { kind: DocComment(Line(Symbol(SymbolIndex { idx: 42 }))), span: Span { base_or_index: 1, len_or_tag: 6 } }
        Token { kind: Newline, span: Span { base_or_index: 7, len_or_tag: 1 } }
        Token { kind: Eof, span: Span { base_or_index: 8, len_or_tag: 0 } }
"#]],
    )
}

#[test]
fn simple_tokens() {
    check_lexing(
        r####"
,
.
(
)
{
}
[
]
@
#
~
?
:
$
=
!
<
>
==
!=
>=
<=
-
&
|
+
*
/
^
%
**
//
<<
>>
...
+=
-=
*=
/=
%=
&=
|=
^=
**=
//=
<<=
>>=
->
"####,
        expect![[r#"
            Token { kind: Newline, span: Span { base_or_index: 0, len_or_tag: 1 } }
            Token { kind: Comma, span: Span { base_or_index: 1, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 2, len_or_tag: 1 } }
            Token { kind: Dot, span: Span { base_or_index: 3, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 4, len_or_tag: 1 } }
            Token { kind: OpenDelim(Paren), span: Span { base_or_index: 5, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 6, len_or_tag: 1 } }
            Token { kind: CloseDelim(Paren), span: Span { base_or_index: 7, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 8, len_or_tag: 1 } }
            Token { kind: OpenDelim(Brace), span: Span { base_or_index: 9, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 10, len_or_tag: 1 } }
            Token { kind: CloseDelim(Brace), span: Span { base_or_index: 11, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 12, len_or_tag: 1 } }
            Token { kind: OpenDelim(Bracket), span: Span { base_or_index: 13, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 14, len_or_tag: 1 } }
            Token { kind: CloseDelim(Bracket), span: Span { base_or_index: 15, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 16, len_or_tag: 1 } }
            Token { kind: At, span: Span { base_or_index: 17, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 18, len_or_tag: 1 } }
            Token { kind: DocComment(Line(Symbol(SymbolIndex { idx: 42 }))), span: Span { base_or_index: 19, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 20, len_or_tag: 1 } }
            Token { kind: UnaryOp(UTilde), span: Span { base_or_index: 21, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 22, len_or_tag: 1 } }
            Token { kind: Question, span: Span { base_or_index: 23, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 24, len_or_tag: 1 } }
            Token { kind: Colon, span: Span { base_or_index: 25, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 26, len_or_tag: 1 } }
            Token { kind: Dollar, span: Span { base_or_index: 27, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 28, len_or_tag: 1 } }
            Token { kind: Assign, span: Span { base_or_index: 29, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 30, len_or_tag: 1 } }
            Token { kind: UnaryOp(UNot), span: Span { base_or_index: 31, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 32, len_or_tag: 1 } }
            Token { kind: BinCmp(Lt), span: Span { base_or_index: 33, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 34, len_or_tag: 1 } }
            Token { kind: BinCmp(Gt), span: Span { base_or_index: 35, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 36, len_or_tag: 1 } }
            Token { kind: BinCmp(Eq), span: Span { base_or_index: 37, len_or_tag: 2 } }
            Token { kind: Newline, span: Span { base_or_index: 39, len_or_tag: 1 } }
            Token { kind: BinCmp(NotEq), span: Span { base_or_index: 40, len_or_tag: 2 } }
            Token { kind: Newline, span: Span { base_or_index: 42, len_or_tag: 1 } }
            Token { kind: BinCmp(GtEq), span: Span { base_or_index: 43, len_or_tag: 2 } }
            Token { kind: Newline, span: Span { base_or_index: 45, len_or_tag: 1 } }
            Token { kind: BinCmp(LtEq), span: Span { base_or_index: 46, len_or_tag: 2 } }
            Token { kind: Newline, span: Span { base_or_index: 48, len_or_tag: 1 } }
            Token { kind: BinOp(Minus), span: Span { base_or_index: 49, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 50, len_or_tag: 1 } }
            Token { kind: BinOp(And), span: Span { base_or_index: 51, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 52, len_or_tag: 1 } }
            Token { kind: BinOp(Or), span: Span { base_or_index: 53, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 54, len_or_tag: 1 } }
            Token { kind: BinOp(Plus), span: Span { base_or_index: 55, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 56, len_or_tag: 1 } }
            Token { kind: BinOp(Star), span: Span { base_or_index: 57, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 58, len_or_tag: 1 } }
            Token { kind: BinOp(Slash), span: Span { base_or_index: 59, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 60, len_or_tag: 1 } }
            Token { kind: BinOp(Caret), span: Span { base_or_index: 61, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 62, len_or_tag: 1 } }
            Token { kind: BinOp(Percent), span: Span { base_or_index: 63, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 64, len_or_tag: 1 } }
            Token { kind: BinOp(StarStar), span: Span { base_or_index: 65, len_or_tag: 2 } }
            Token { kind: Newline, span: Span { base_or_index: 67, len_or_tag: 1 } }
            Token { kind: BinOp(SlashSlash), span: Span { base_or_index: 68, len_or_tag: 2 } }
            Token { kind: Newline, span: Span { base_or_index: 70, len_or_tag: 1 } }
            Token { kind: BinOp(Shl), span: Span { base_or_index: 71, len_or_tag: 2 } }
            Token { kind: Newline, span: Span { base_or_index: 73, len_or_tag: 1 } }
            Token { kind: BinOp(Shr), span: Span { base_or_index: 74, len_or_tag: 2 } }
            Token { kind: Newline, span: Span { base_or_index: 76, len_or_tag: 1 } }
            Token { kind: DotDotDot, span: Span { base_or_index: 77, len_or_tag: 3 } }
            Token { kind: Newline, span: Span { base_or_index: 80, len_or_tag: 1 } }
            Token { kind: BinOpEq(Plus), span: Span { base_or_index: 81, len_or_tag: 2 } }
            Token { kind: Newline, span: Span { base_or_index: 83, len_or_tag: 1 } }
            Token { kind: BinOpEq(Minus), span: Span { base_or_index: 84, len_or_tag: 2 } }
            Token { kind: Newline, span: Span { base_or_index: 86, len_or_tag: 1 } }
            Token { kind: BinOpEq(Star), span: Span { base_or_index: 87, len_or_tag: 2 } }
            Token { kind: Newline, span: Span { base_or_index: 89, len_or_tag: 1 } }
            Token { kind: BinOpEq(Slash), span: Span { base_or_index: 90, len_or_tag: 2 } }
            Token { kind: Newline, span: Span { base_or_index: 92, len_or_tag: 1 } }
            Token { kind: BinOpEq(Percent), span: Span { base_or_index: 93, len_or_tag: 2 } }
            Token { kind: Newline, span: Span { base_or_index: 95, len_or_tag: 1 } }
            Token { kind: BinOpEq(And), span: Span { base_or_index: 96, len_or_tag: 2 } }
            Token { kind: Newline, span: Span { base_or_index: 98, len_or_tag: 1 } }
            Token { kind: BinOpEq(Or), span: Span { base_or_index: 99, len_or_tag: 2 } }
            Token { kind: Newline, span: Span { base_or_index: 101, len_or_tag: 1 } }
            Token { kind: BinOpEq(Caret), span: Span { base_or_index: 102, len_or_tag: 2 } }
            Token { kind: Newline, span: Span { base_or_index: 104, len_or_tag: 1 } }
            Token { kind: BinOpEq(StarStar), span: Span { base_or_index: 105, len_or_tag: 3 } }
            Token { kind: Newline, span: Span { base_or_index: 108, len_or_tag: 1 } }
            Token { kind: BinOpEq(SlashSlash), span: Span { base_or_index: 109, len_or_tag: 3 } }
            Token { kind: Newline, span: Span { base_or_index: 112, len_or_tag: 1 } }
            Token { kind: BinOpEq(Shl), span: Span { base_or_index: 113, len_or_tag: 3 } }
            Token { kind: Newline, span: Span { base_or_index: 116, len_or_tag: 1 } }
            Token { kind: BinOpEq(Shr), span: Span { base_or_index: 117, len_or_tag: 3 } }
            Token { kind: Newline, span: Span { base_or_index: 120, len_or_tag: 1 } }
            Token { kind: RArrow, span: Span { base_or_index: 121, len_or_tag: 2 } }
            Token { kind: Newline, span: Span { base_or_index: 123, len_or_tag: 1 } }
            Token { kind: Eof, span: Span { base_or_index: 124, len_or_tag: 0 } }
        "#]],
    )
}

#[test]
fn nonstring_literal() {
    check_lexing(
        r####"
1234
0b101
0xABC
1.0
1.0e10
0777
0077
1Ki
"####,
        expect![[r#"
            Token { kind: Newline, span: Span { base_or_index: 0, len_or_tag: 1 } }
            Token { kind: Literal(Lit { kind: Integer, symbol: Symbol(SymbolIndex { idx: 42 }), suffix: None, raw: None }), span: Span { base_or_index: 1, len_or_tag: 4 } }
            Token { kind: Newline, span: Span { base_or_index: 5, len_or_tag: 1 } }
            Token { kind: Literal(Lit { kind: Integer, symbol: Symbol(SymbolIndex { idx: 43 }), suffix: None, raw: None }), span: Span { base_or_index: 6, len_or_tag: 5 } }
            Token { kind: Newline, span: Span { base_or_index: 11, len_or_tag: 1 } }
            Token { kind: Literal(Lit { kind: Integer, symbol: Symbol(SymbolIndex { idx: 44 }), suffix: None, raw: None }), span: Span { base_or_index: 12, len_or_tag: 5 } }
            Token { kind: Newline, span: Span { base_or_index: 17, len_or_tag: 1 } }
            Token { kind: Literal(Lit { kind: Float, symbol: Symbol(SymbolIndex { idx: 45 }), suffix: None, raw: None }), span: Span { base_or_index: 18, len_or_tag: 3 } }
            Token { kind: Newline, span: Span { base_or_index: 21, len_or_tag: 1 } }
            Token { kind: Literal(Lit { kind: Float, symbol: Symbol(SymbolIndex { idx: 46 }), suffix: None, raw: None }), span: Span { base_or_index: 22, len_or_tag: 6 } }
            Token { kind: Newline, span: Span { base_or_index: 28, len_or_tag: 1 } }
            Token { kind: Literal(Lit { kind: Integer, symbol: Symbol(SymbolIndex { idx: 47 }), suffix: None, raw: None }), span: Span { base_or_index: 29, len_or_tag: 4 } }
            Token { kind: Newline, span: Span { base_or_index: 33, len_or_tag: 1 } }
            Token { kind: Literal(Lit { kind: Integer, symbol: Symbol(SymbolIndex { idx: 48 }), suffix: None, raw: None }), span: Span { base_or_index: 34, len_or_tag: 4 } }
            Token { kind: Newline, span: Span { base_or_index: 38, len_or_tag: 1 } }
            Token { kind: Literal(Lit { kind: Integer, symbol: Symbol(SymbolIndex { idx: 33 }), suffix: Some(Symbol(SymbolIndex { idx: 49 })), raw: None }), span: Span { base_or_index: 39, len_or_tag: 3 } }
            Token { kind: Newline, span: Span { base_or_index: 42, len_or_tag: 1 } }
            Token { kind: Eof, span: Span { base_or_index: 43, len_or_tag: 0 } }
        "#]],
    )
}

#[test]
fn string_literal() {
    check_lexing(
        r####"
'a'
"a"
'''a'''
"""a"""
r'a'
r"a"
r'''a'''
r"""a"""
R'a'
R"a"
R'''a'''
R"""a"""
"####,
        expect![[r#"
        Token { kind: Newline, span: Span { base_or_index: 0, len_or_tag: 1 } }
        Token { kind: Literal(Lit { kind: Str { is_long_string: false, is_raw: false }, symbol: Symbol(SymbolIndex { idx: 42 }), suffix: None, raw: Some(Symbol(SymbolIndex { idx: 43 })) }), span: Span { base_or_index: 1, len_or_tag: 3 } }
        Token { kind: Newline, span: Span { base_or_index: 4, len_or_tag: 1 } }
        Token { kind: Literal(Lit { kind: Str { is_long_string: false, is_raw: false }, symbol: Symbol(SymbolIndex { idx: 42 }), suffix: None, raw: Some(Symbol(SymbolIndex { idx: 44 })) }), span: Span { base_or_index: 5, len_or_tag: 3 } }
        Token { kind: Newline, span: Span { base_or_index: 8, len_or_tag: 1 } }
        Token { kind: Literal(Lit { kind: Str { is_long_string: true, is_raw: false }, symbol: Symbol(SymbolIndex { idx: 42 }), suffix: None, raw: Some(Symbol(SymbolIndex { idx: 45 })) }), span: Span { base_or_index: 9, len_or_tag: 7 } }
        Token { kind: Newline, span: Span { base_or_index: 16, len_or_tag: 1 } }
        Token { kind: Literal(Lit { kind: Str { is_long_string: true, is_raw: false }, symbol: Symbol(SymbolIndex { idx: 42 }), suffix: None, raw: Some(Symbol(SymbolIndex { idx: 46 })) }), span: Span { base_or_index: 17, len_or_tag: 7 } }
        Token { kind: Newline, span: Span { base_or_index: 24, len_or_tag: 1 } }
        Token { kind: Literal(Lit { kind: Str { is_long_string: false, is_raw: true }, symbol: Symbol(SymbolIndex { idx: 42 }), suffix: None, raw: Some(Symbol(SymbolIndex { idx: 47 })) }), span: Span { base_or_index: 25, len_or_tag: 4 } }
        Token { kind: Newline, span: Span { base_or_index: 29, len_or_tag: 1 } }
        Token { kind: Literal(Lit { kind: Str { is_long_string: false, is_raw: true }, symbol: Symbol(SymbolIndex { idx: 42 }), suffix: None, raw: Some(Symbol(SymbolIndex { idx: 48 })) }), span: Span { base_or_index: 30, len_or_tag: 4 } }
        Token { kind: Newline, span: Span { base_or_index: 34, len_or_tag: 1 } }
        Token { kind: Literal(Lit { kind: Str { is_long_string: true, is_raw: true }, symbol: Symbol(SymbolIndex { idx: 42 }), suffix: None, raw: Some(Symbol(SymbolIndex { idx: 49 })) }), span: Span { base_or_index: 35, len_or_tag: 8 } }
        Token { kind: Newline, span: Span { base_or_index: 43, len_or_tag: 1 } }
        Token { kind: Literal(Lit { kind: Str { is_long_string: true, is_raw: true }, symbol: Symbol(SymbolIndex { idx: 42 }), suffix: None, raw: Some(Symbol(SymbolIndex { idx: 50 })) }), span: Span { base_or_index: 44, len_or_tag: 8 } }
        Token { kind: Newline, span: Span { base_or_index: 52, len_or_tag: 1 } }
        Token { kind: Literal(Lit { kind: Str { is_long_string: false, is_raw: true }, symbol: Symbol(SymbolIndex { idx: 42 }), suffix: None, raw: Some(Symbol(SymbolIndex { idx: 51 })) }), span: Span { base_or_index: 53, len_or_tag: 4 } }
        Token { kind: Newline, span: Span { base_or_index: 57, len_or_tag: 1 } }
        Token { kind: Literal(Lit { kind: Str { is_long_string: false, is_raw: true }, symbol: Symbol(SymbolIndex { idx: 42 }), suffix: None, raw: Some(Symbol(SymbolIndex { idx: 52 })) }), span: Span { base_or_index: 58, len_or_tag: 4 } }
        Token { kind: Newline, span: Span { base_or_index: 62, len_or_tag: 1 } }
        Token { kind: Literal(Lit { kind: Str { is_long_string: true, is_raw: true }, symbol: Symbol(SymbolIndex { idx: 42 }), suffix: None, raw: Some(Symbol(SymbolIndex { idx: 53 })) }), span: Span { base_or_index: 63, len_or_tag: 8 } }
        Token { kind: Newline, span: Span { base_or_index: 71, len_or_tag: 1 } }
        Token { kind: Literal(Lit { kind: Str { is_long_string: true, is_raw: true }, symbol: Symbol(SymbolIndex { idx: 42 }), suffix: None, raw: Some(Symbol(SymbolIndex { idx: 54 })) }), span: Span { base_or_index: 72, len_or_tag: 8 } }
        Token { kind: Newline, span: Span { base_or_index: 80, len_or_tag: 1 } }
        Token { kind: Eof, span: Span { base_or_index: 81, len_or_tag: 0 } }
        "#]],
    )
}

#[test]
fn indents() {
    check_lexing(
        r####"
if test0:
    if test1:
        println("true true")
    else:
        println("true false")
println("end")
"####,
        expect![[r#"
            Token { kind: Newline, span: Span { base_or_index: 0, len_or_tag: 1 } }
            Token { kind: Ident(Symbol(SymbolIndex { idx: 10 })), span: Span { base_or_index: 1, len_or_tag: 2 } }
            Token { kind: Ident(Symbol(SymbolIndex { idx: 42 })), span: Span { base_or_index: 4, len_or_tag: 5 } }
            Token { kind: Colon, span: Span { base_or_index: 9, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 10, len_or_tag: 1 } }
            Token { kind: Indent(0), span: Span { base_or_index: 15, len_or_tag: 0 } }
            Token { kind: Ident(Symbol(SymbolIndex { idx: 10 })), span: Span { base_or_index: 15, len_or_tag: 2 } }
            Token { kind: Ident(Symbol(SymbolIndex { idx: 43 })), span: Span { base_or_index: 18, len_or_tag: 5 } }
            Token { kind: Colon, span: Span { base_or_index: 23, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 24, len_or_tag: 1 } }
            Token { kind: Indent(0), span: Span { base_or_index: 33, len_or_tag: 0 } }
            Token { kind: Ident(Symbol(SymbolIndex { idx: 44 })), span: Span { base_or_index: 33, len_or_tag: 7 } }
            Token { kind: OpenDelim(Paren), span: Span { base_or_index: 40, len_or_tag: 1 } }
            Token { kind: Literal(Lit { kind: Str { is_long_string: false, is_raw: false }, symbol: Symbol(SymbolIndex { idx: 45 }), suffix: None, raw: Some(Symbol(SymbolIndex { idx: 46 })) }), span: Span { base_or_index: 41, len_or_tag: 11 } }
            Token { kind: CloseDelim(Paren), span: Span { base_or_index: 52, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 53, len_or_tag: 1 } }
            Token { kind: Dedent(0), span: Span { base_or_index: 58, len_or_tag: 0 } }
            Token { kind: Ident(Symbol(SymbolIndex { idx: 12 })), span: Span { base_or_index: 58, len_or_tag: 4 } }
            Token { kind: Colon, span: Span { base_or_index: 62, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 63, len_or_tag: 1 } }
            Token { kind: Indent(0), span: Span { base_or_index: 72, len_or_tag: 0 } }
            Token { kind: Ident(Symbol(SymbolIndex { idx: 44 })), span: Span { base_or_index: 72, len_or_tag: 7 } }
            Token { kind: OpenDelim(Paren), span: Span { base_or_index: 79, len_or_tag: 1 } }
            Token { kind: Literal(Lit { kind: Str { is_long_string: false, is_raw: false }, symbol: Symbol(SymbolIndex { idx: 47 }), suffix: None, raw: Some(Symbol(SymbolIndex { idx: 48 })) }), span: Span { base_or_index: 80, len_or_tag: 12 } }
            Token { kind: CloseDelim(Paren), span: Span { base_or_index: 92, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 93, len_or_tag: 1 } }
            Token { kind: Dedent(0), span: Span { base_or_index: 94, len_or_tag: 0 } }
            Token { kind: Dedent(0), span: Span { base_or_index: 94, len_or_tag: 0 } }
            Token { kind: Ident(Symbol(SymbolIndex { idx: 44 })), span: Span { base_or_index: 94, len_or_tag: 7 } }
            Token { kind: OpenDelim(Paren), span: Span { base_or_index: 101, len_or_tag: 1 } }
            Token { kind: Literal(Lit { kind: Str { is_long_string: false, is_raw: false }, symbol: Symbol(SymbolIndex { idx: 49 }), suffix: None, raw: Some(Symbol(SymbolIndex { idx: 50 })) }), span: Span { base_or_index: 102, len_or_tag: 5 } }
            Token { kind: CloseDelim(Paren), span: Span { base_or_index: 107, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 108, len_or_tag: 1 } }
            Token { kind: Eof, span: Span { base_or_index: 109, len_or_tag: 0 } }
        "#]],
    )
}

#[test]
fn binary_expr_0() {
    check_lexing(
        r####"1 + a or b"####,
        expect![[r#"
        Token { kind: Literal(Lit { kind: Integer, symbol: Symbol(SymbolIndex { idx: 33 }), suffix: None, raw: None }), span: Span { base_or_index: 0, len_or_tag: 1 } }
        Token { kind: BinOp(Plus), span: Span { base_or_index: 2, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 42 })), span: Span { base_or_index: 4, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 13 })), span: Span { base_or_index: 6, len_or_tag: 2 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 43 })), span: Span { base_or_index: 9, len_or_tag: 1 } }
        Token { kind: Newline, span: Span { base_or_index: 10, len_or_tag: 0 } }
        Token { kind: Eof, span: Span { base_or_index: 10, len_or_tag: 0 } }
        "#]],
    );
}

#[test]
fn schema_expr_0() {
    check_lexing(
        r####"
Schema (1, 2) {
    k=v
}
"####,
        expect![[r#"
        Token { kind: Newline, span: Span { base_or_index: 0, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 42 })), span: Span { base_or_index: 1, len_or_tag: 6 } }
        Token { kind: OpenDelim(Paren), span: Span { base_or_index: 8, len_or_tag: 1 } }
        Token { kind: Literal(Lit { kind: Integer, symbol: Symbol(SymbolIndex { idx: 33 }), suffix: None, raw: None }), span: Span { base_or_index: 9, len_or_tag: 1 } }
        Token { kind: Comma, span: Span { base_or_index: 10, len_or_tag: 1 } }
        Token { kind: Literal(Lit { kind: Integer, symbol: Symbol(SymbolIndex { idx: 34 }), suffix: None, raw: None }), span: Span { base_or_index: 12, len_or_tag: 1 } }
        Token { kind: CloseDelim(Paren), span: Span { base_or_index: 13, len_or_tag: 1 } }
        Token { kind: OpenDelim(Brace), span: Span { base_or_index: 15, len_or_tag: 1 } }
        Token { kind: Newline, span: Span { base_or_index: 16, len_or_tag: 1 } }
        Token { kind: Indent(0), span: Span { base_or_index: 21, len_or_tag: 0 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 43 })), span: Span { base_or_index: 21, len_or_tag: 1 } }
        Token { kind: Assign, span: Span { base_or_index: 22, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 44 })), span: Span { base_or_index: 23, len_or_tag: 1 } }
        Token { kind: Newline, span: Span { base_or_index: 24, len_or_tag: 1 } }
        Token { kind: Dedent(0), span: Span { base_or_index: 25, len_or_tag: 0 } }
        Token { kind: CloseDelim(Brace), span: Span { base_or_index: 25, len_or_tag: 1 } }
        Token { kind: Newline, span: Span { base_or_index: 26, len_or_tag: 1 } }
        Token { kind: Eof, span: Span { base_or_index: 27, len_or_tag: 0 } }
        "#]],
    );
}

#[test]
fn schema_expr_1() {
    check_lexing(
        r####"Schema (1, 2) {
    k=v
}"####,
        expect![[r#"
        Token { kind: Ident(Symbol(SymbolIndex { idx: 42 })), span: Span { base_or_index: 0, len_or_tag: 6 } }
        Token { kind: OpenDelim(Paren), span: Span { base_or_index: 7, len_or_tag: 1 } }
        Token { kind: Literal(Lit { kind: Integer, symbol: Symbol(SymbolIndex { idx: 33 }), suffix: None, raw: None }), span: Span { base_or_index: 8, len_or_tag: 1 } }
        Token { kind: Comma, span: Span { base_or_index: 9, len_or_tag: 1 } }
        Token { kind: Literal(Lit { kind: Integer, symbol: Symbol(SymbolIndex { idx: 34 }), suffix: None, raw: None }), span: Span { base_or_index: 11, len_or_tag: 1 } }
        Token { kind: CloseDelim(Paren), span: Span { base_or_index: 12, len_or_tag: 1 } }
        Token { kind: OpenDelim(Brace), span: Span { base_or_index: 14, len_or_tag: 1 } }
        Token { kind: Newline, span: Span { base_or_index: 15, len_or_tag: 1 } }
        Token { kind: Indent(0), span: Span { base_or_index: 20, len_or_tag: 0 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 43 })), span: Span { base_or_index: 20, len_or_tag: 1 } }
        Token { kind: Assign, span: Span { base_or_index: 21, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 44 })), span: Span { base_or_index: 22, len_or_tag: 1 } }
        Token { kind: Newline, span: Span { base_or_index: 23, len_or_tag: 1 } }
        Token { kind: Dedent(0), span: Span { base_or_index: 24, len_or_tag: 0 } }
        Token { kind: CloseDelim(Brace), span: Span { base_or_index: 24, len_or_tag: 1 } }
        Token { kind: Newline, span: Span { base_or_index: 25, len_or_tag: 0 } }
        Token { kind: Eof, span: Span { base_or_index: 25, len_or_tag: 0 } }
        "#]],
    );
}

#[test]
fn test_peek() {
    let src = "\na=1";
    let sm = SourceMap::new(FilePathMapping::empty());
    sm.new_source_file(PathBuf::from("").into(), src.to_string());
    let sess = ParseSession::with_source_map(Arc::new(sm));

    create_session_globals_then(|| {
        let stream = parse_token_streams(&sess, src, new_byte_pos(0));
        let mut cursor = stream.cursor();

        let tok0 = cursor.next();
        assert_eq!(
            format!("{tok0:?}"),
            "Some(Token { kind: Newline, span: Span { base_or_index: 0, len_or_tag: 1 } })"
        );

        let peek = cursor.peek();
        assert_eq!(
            format!("{peek:?}"),
           "Some(Token { kind: Ident(Symbol(SymbolIndex { idx: 42 })), span: Span { base_or_index: 1, len_or_tag: 1 } })"
        );
    });
}

#[test]
fn test_assign_stmt() {
    check_lexing(
        r####"
a=1
"####,
        expect![[r#"
        Token { kind: Newline, span: Span { base_or_index: 0, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 42 })), span: Span { base_or_index: 1, len_or_tag: 1 } }
        Token { kind: Assign, span: Span { base_or_index: 2, len_or_tag: 1 } }
        Token { kind: Literal(Lit { kind: Integer, symbol: Symbol(SymbolIndex { idx: 33 }), suffix: None, raw: None }), span: Span { base_or_index: 3, len_or_tag: 1 } }
        Token { kind: Newline, span: Span { base_or_index: 4, len_or_tag: 1 } }
        Token { kind: Eof, span: Span { base_or_index: 5, len_or_tag: 0 } }
        "#]],
    )
}

#[test]
fn test_token_span() {
    let src = r#"
schema Person:
    name: str = "kcl"

x0 = Person {}
    "#;
    check_span(
        src,
        expect![
            r#""\n"
"schema"
"Person"
":"
"\n"
""
"name"
":"
"str"
"="
"\"kcl\""
"\n"
"\n"
""
"x0"
"="
"Person"
"{"
"}"
"\n"
""
"#
        ],
    )
}

#[test]
fn test_source_file() {
    let src = "\r\n\r\n\r\r\n\n\n\r".to_string();
    let sm = kclvm_span::SourceMap::new(FilePathMapping::empty());
    let sf = sm.new_source_file(PathBuf::from("").into(), src);
    match sf.src.as_ref() {
        Some(src_from_sf) => {
            assert_eq!(src_from_sf.as_str(), "\n\n\r\n\n\n\r");
        }
        None => {
            unreachable!();
        }
    };
}

#[test]
fn test_parse_token_stream() {
    check_lexing(
        "\n\r\n\r\n\r\r\n",
        expect![[r#"
            Token { kind: Newline, span: Span { base_or_index: 0, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 1, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 2, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 4, len_or_tag: 1 } }
            Token { kind: Eof, span: Span { base_or_index: 5, len_or_tag: 0 } }
        "#]],
    );
}

#[cfg(target_os = "windows")]
#[test]
fn test_parse_token_stream_on_win() {
    use std::{fs, path::Path};
    let src = fs::read_to_string(
        Path::new(".")
            .join("testdata")
            .join("hello_win.k")
            .display()
            .to_string(),
    )
    .unwrap();
    assert_eq!(
        src,
        "\r\nschema Person:\r\n    name: str = \"kcl\"\r\n\r\nx0 = Person {}\r\n"
    );

    check_lexing(
        &src,
        expect![[r#"
            Token { kind: Newline, span: Span { base_or_index: 0, len_or_tag: 1 } }
            Token { kind: Ident(Symbol(SymbolIndex { idx: 4 })), span: Span { base_or_index: 1, len_or_tag: 6 } }
            Token { kind: Ident(Symbol(SymbolIndex { idx: 42 })), span: Span { base_or_index: 8, len_or_tag: 6 } }
            Token { kind: Colon, span: Span { base_or_index: 14, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 15, len_or_tag: 1 } }
            Token { kind: Indent(0), span: Span { base_or_index: 20, len_or_tag: 0 } }
            Token { kind: Ident(Symbol(SymbolIndex { idx: 43 })), span: Span { base_or_index: 20, len_or_tag: 4 } }
            Token { kind: Colon, span: Span { base_or_index: 24, len_or_tag: 1 } }
            Token { kind: Ident(Symbol(SymbolIndex { idx: 31 })), span: Span { base_or_index: 26, len_or_tag: 3 } }
            Token { kind: Assign, span: Span { base_or_index: 30, len_or_tag: 1 } }
            Token { kind: Literal(Lit { kind: Str { is_long_string: false, is_raw: false }, symbol: Symbol(SymbolIndex { idx: 44 }), suffix: None, raw: Some(Symbol(SymbolIndex { idx: 45 })) }), span: Span { base_or_index: 32, len_or_tag: 5 } }
            Token { kind: Newline, span: Span { base_or_index: 37, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 38, len_or_tag: 1 } }
            Token { kind: Dedent(0), span: Span { base_or_index: 39, len_or_tag: 0 } }
            Token { kind: Ident(Symbol(SymbolIndex { idx: 46 })), span: Span { base_or_index: 39, len_or_tag: 2 } }
            Token { kind: Assign, span: Span { base_or_index: 42, len_or_tag: 1 } }
            Token { kind: Ident(Symbol(SymbolIndex { idx: 42 })), span: Span { base_or_index: 44, len_or_tag: 6 } }
            Token { kind: OpenDelim(Brace), span: Span { base_or_index: 51, len_or_tag: 1 } }
            Token { kind: CloseDelim(Brace), span: Span { base_or_index: 52, len_or_tag: 1 } }
            Token { kind: Newline, span: Span { base_or_index: 53, len_or_tag: 1 } }
            Token { kind: Eof, span: Span { base_or_index: 54, len_or_tag: 0 } }
        "#]],
    );
}

#[test]
fn test_rarrow() {
    check_lexing(
        "lambda x: int, y: int -> int { x + y }\n",
        expect![[r#"
        Token { kind: Ident(Symbol(SymbolIndex { idx: 18 })), span: Span { base_or_index: 0, len_or_tag: 6 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 42 })), span: Span { base_or_index: 7, len_or_tag: 1 } }
        Token { kind: Colon, span: Span { base_or_index: 8, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 30 })), span: Span { base_or_index: 10, len_or_tag: 3 } }
        Token { kind: Comma, span: Span { base_or_index: 13, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 43 })), span: Span { base_or_index: 15, len_or_tag: 1 } }
        Token { kind: Colon, span: Span { base_or_index: 16, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 30 })), span: Span { base_or_index: 18, len_or_tag: 3 } }
        Token { kind: RArrow, span: Span { base_or_index: 22, len_or_tag: 2 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 30 })), span: Span { base_or_index: 25, len_or_tag: 3 } }
        Token { kind: OpenDelim(Brace), span: Span { base_or_index: 29, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 42 })), span: Span { base_or_index: 31, len_or_tag: 1 } }
        Token { kind: BinOp(Plus), span: Span { base_or_index: 33, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 43 })), span: Span { base_or_index: 35, len_or_tag: 1 } }
        Token { kind: CloseDelim(Brace), span: Span { base_or_index: 37, len_or_tag: 1 } }
        Token { kind: Newline, span: Span { base_or_index: 38, len_or_tag: 1 } }
        Token { kind: Eof, span: Span { base_or_index: 39, len_or_tag: 0 } }
        "#]],
    );
}

#[test]
fn test_minus_unicode_gt_invalid() {
    check_lexing_with_err_msg(
        "lambda x: int, y: int -\u{feff}> int { x + y }\n",
        expect![[r#"
        Token { kind: Ident(Symbol(SymbolIndex { idx: 18 })), span: Span { base_or_index: 0, len_or_tag: 6 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 42 })), span: Span { base_or_index: 7, len_or_tag: 1 } }
        Token { kind: Colon, span: Span { base_or_index: 8, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 30 })), span: Span { base_or_index: 10, len_or_tag: 3 } }
        Token { kind: Comma, span: Span { base_or_index: 13, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 43 })), span: Span { base_or_index: 15, len_or_tag: 1 } }
        Token { kind: Colon, span: Span { base_or_index: 16, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 30 })), span: Span { base_or_index: 18, len_or_tag: 3 } }
        Token { kind: RArrow, span: Span { base_or_index: 25, len_or_tag: 2 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 30 })), span: Span { base_or_index: 28, len_or_tag: 3 } }
        Token { kind: OpenDelim(Brace), span: Span { base_or_index: 32, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 42 })), span: Span { base_or_index: 34, len_or_tag: 1 } }
        Token { kind: BinOp(Plus), span: Span { base_or_index: 36, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 43 })), span: Span { base_or_index: 38, len_or_tag: 1 } }
        Token { kind: CloseDelim(Brace), span: Span { base_or_index: 40, len_or_tag: 1 } }
        Token { kind: Newline, span: Span { base_or_index: 41, len_or_tag: 1 } }
        Token { kind: Eof, span: Span { base_or_index: 42, len_or_tag: 0 } }
        "#]],
        expect![["error[E1001]: InvalidSyntax\nunknown start of token\n\n"]],
    );
}

#[test]
fn test_unicode_minus_gt_invalid() {
    check_lexing_with_err_msg(
        "lambda x: int, y: int \u{feff}-> int { x + y }\n",
        expect![[r#"
        Token { kind: Ident(Symbol(SymbolIndex { idx: 18 })), span: Span { base_or_index: 0, len_or_tag: 6 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 42 })), span: Span { base_or_index: 7, len_or_tag: 1 } }
        Token { kind: Colon, span: Span { base_or_index: 8, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 30 })), span: Span { base_or_index: 10, len_or_tag: 3 } }
        Token { kind: Comma, span: Span { base_or_index: 13, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 43 })), span: Span { base_or_index: 15, len_or_tag: 1 } }
        Token { kind: Colon, span: Span { base_or_index: 16, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 30 })), span: Span { base_or_index: 18, len_or_tag: 3 } }
        Token { kind: RArrow, span: Span { base_or_index: 25, len_or_tag: 2 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 30 })), span: Span { base_or_index: 28, len_or_tag: 3 } }
        Token { kind: OpenDelim(Brace), span: Span { base_or_index: 32, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 42 })), span: Span { base_or_index: 34, len_or_tag: 1 } }
        Token { kind: BinOp(Plus), span: Span { base_or_index: 36, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 43 })), span: Span { base_or_index: 38, len_or_tag: 1 } }
        Token { kind: CloseDelim(Brace), span: Span { base_or_index: 40, len_or_tag: 1 } }
        Token { kind: Newline, span: Span { base_or_index: 41, len_or_tag: 1 } }
        Token { kind: Eof, span: Span { base_or_index: 42, len_or_tag: 0 } }
        "#]],
        expect![["error[E1001]: InvalidSyntax\nunknown start of token\n\n"]],
    );
}

#[test]
fn test_minus_gt_unicode_invalid() {
    check_lexing_with_err_msg(
        "lambda x: int, y: int ->\u{feff} int { x + y }\n",
        expect![[r#"
        Token { kind: Ident(Symbol(SymbolIndex { idx: 18 })), span: Span { base_or_index: 0, len_or_tag: 6 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 42 })), span: Span { base_or_index: 7, len_or_tag: 1 } }
        Token { kind: Colon, span: Span { base_or_index: 8, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 30 })), span: Span { base_or_index: 10, len_or_tag: 3 } }
        Token { kind: Comma, span: Span { base_or_index: 13, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 43 })), span: Span { base_or_index: 15, len_or_tag: 1 } }
        Token { kind: Colon, span: Span { base_or_index: 16, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 30 })), span: Span { base_or_index: 18, len_or_tag: 3 } }
        Token { kind: RArrow, span: Span { base_or_index: 22, len_or_tag: 2 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 30 })), span: Span { base_or_index: 28, len_or_tag: 3 } }
        Token { kind: OpenDelim(Brace), span: Span { base_or_index: 32, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 42 })), span: Span { base_or_index: 34, len_or_tag: 1 } }
        Token { kind: BinOp(Plus), span: Span { base_or_index: 36, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 43 })), span: Span { base_or_index: 38, len_or_tag: 1 } }
        Token { kind: CloseDelim(Brace), span: Span { base_or_index: 40, len_or_tag: 1 } }
        Token { kind: Newline, span: Span { base_or_index: 41, len_or_tag: 1 } }
        Token { kind: Eof, span: Span { base_or_index: 42, len_or_tag: 0 } }
        "#]],
        expect![["error[E1001]: InvalidSyntax\nunknown start of token\n\n"]],
    );
}

#[test]
fn test_only_minus() {
    check_lexing(
        "-",
        expect![[r#"
        Token { kind: BinOp(Minus), span: Span { base_or_index: 0, len_or_tag: 1 } }
        Token { kind: Newline, span: Span { base_or_index: 1, len_or_tag: 0 } }
        Token { kind: Eof, span: Span { base_or_index: 1, len_or_tag: 0 } }
        "#]],
    );
}

#[test]
fn test_begin_with_minus() {
    check_lexing(
        "-123",
        expect![[r#"
        Token { kind: BinOp(Minus), span: Span { base_or_index: 0, len_or_tag: 1 } }
        Token { kind: Literal(Lit { kind: Integer, symbol: Symbol(SymbolIndex { idx: 42 }), suffix: None, raw: None }), span: Span { base_or_index: 1, len_or_tag: 3 } }
        Token { kind: Newline, span: Span { base_or_index: 4, len_or_tag: 0 } }
        Token { kind: Eof, span: Span { base_or_index: 4, len_or_tag: 0 } }
        "#]],
    );
}

#[test]
fn test_only_gt() {
    check_lexing(
        ">",
        expect![[r#"
        Token { kind: BinCmp(Gt), span: Span { base_or_index: 0, len_or_tag: 1 } }
        Token { kind: Newline, span: Span { base_or_index: 1, len_or_tag: 0 } }
        Token { kind: Eof, span: Span { base_or_index: 1, len_or_tag: 0 } }
        "#]],
    );
}

#[test]
fn test_begin_with_gt() {
    check_lexing(
        ">sdjkd + ==",
        expect![[r#"
        Token { kind: BinCmp(Gt), span: Span { base_or_index: 0, len_or_tag: 1 } }
        Token { kind: Ident(Symbol(SymbolIndex { idx: 42 })), span: Span { base_or_index: 1, len_or_tag: 5 } }
        Token { kind: BinOp(Plus), span: Span { base_or_index: 7, len_or_tag: 1 } }
        Token { kind: BinCmp(Eq), span: Span { base_or_index: 9, len_or_tag: 2 } }
        Token { kind: Newline, span: Span { base_or_index: 11, len_or_tag: 0 } }
        Token { kind: Eof, span: Span { base_or_index: 11, len_or_tag: 0 } }
        "#]],
    );
}

#[test]
fn test_double_rarrow() {
    check_lexing(
        "->->",
        expect![[r#"
        Token { kind: RArrow, span: Span { base_or_index: 0, len_or_tag: 2 } }
        Token { kind: RArrow, span: Span { base_or_index: 2, len_or_tag: 2 } }
        Token { kind: Newline, span: Span { base_or_index: 4, len_or_tag: 0 } }
        Token { kind: Eof, span: Span { base_or_index: 4, len_or_tag: 0 } }
        "#]],
    );
}

#[test]
fn test_mess_rarrow() {
    check_lexing(
        "-->>->",
        expect![[r#"
        Token { kind: BinOp(Minus), span: Span { base_or_index: 0, len_or_tag: 1 } }
        Token { kind: BinOp(Minus), span: Span { base_or_index: 1, len_or_tag: 1 } }
        Token { kind: BinOp(Shr), span: Span { base_or_index: 2, len_or_tag: 2 } }
        Token { kind: RArrow, span: Span { base_or_index: 4, len_or_tag: 2 } }
        Token { kind: Newline, span: Span { base_or_index: 6, len_or_tag: 0 } }
        Token { kind: Eof, span: Span { base_or_index: 6, len_or_tag: 0 } }
        "#]],
    );
}
