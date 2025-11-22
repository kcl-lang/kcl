use super::*;
use expect_test::{expect, Expect};
use std::fmt::Write;

fn check_lexing(src: &str, expect: Expect) {
    let actual: String = tokenize(src).fold(String::new(), |mut acc, token| {
        writeln!(acc, "{:?}", token).expect("Failed to write to string");
        acc
    });
    expect.assert_eq(&actual)
}

#[test]
fn smoke_test() {
    check_lexing(
        "  lambda { println(\"kclvm\"); }\n",
        expect![[r#"
            Token { kind: Space, len: 1 }
            Token { kind: Space, len: 1 }
            Token { kind: Ident, len: 6 }
            Token { kind: Space, len: 1 }
            Token { kind: OpenBrace, len: 1 }
            Token { kind: Space, len: 1 }
            Token { kind: Ident, len: 7 }
            Token { kind: OpenParen, len: 1 }
            Token { kind: Literal { kind: Str { terminated: true, triple_quoted: false }, suffix_start: 7 }, len: 7 }
            Token { kind: CloseParen, len: 1 }
            Token { kind: Semi, len: 1 }
            Token { kind: Space, len: 1 }
            Token { kind: CloseBrace, len: 1 }
            Token { kind: Newline, len: 1 }
        "#]],
    )
}

#[test]
fn comment_flavors() {
    check_lexing(
        r"
#
# line
",
        expect![[r#"
        Token { kind: Newline, len: 1 }
        Token { kind: LineComment { doc_style: Some(Inner) }, len: 1 }
        Token { kind: Newline, len: 1 }
        Token { kind: LineComment { doc_style: Some(Inner) }, len: 6 }
        Token { kind: Newline, len: 1 }
"#]],
    )
}

#[test]
fn simple_tokens() {
    check_lexing(
        r####"
;
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
"####,
        expect![[r#"
            Token { kind: Newline, len: 1 }
            Token { kind: Semi, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: Comma, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: Dot, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: OpenParen, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: CloseParen, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: OpenBrace, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: CloseBrace, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: OpenBracket, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: CloseBracket, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: At, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: LineComment { doc_style: Some(Inner) }, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: Tilde, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: Question, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: Colon, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: Dollar, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: Eq, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: Bang, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: Lt, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: Gt, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: EqEq, len: 2 }
            Token { kind: Newline, len: 1 }
            Token { kind: BangEq, len: 2 }
            Token { kind: Newline, len: 1 }
            Token { kind: GtEq, len: 2 }
            Token { kind: Newline, len: 1 }
            Token { kind: LtEq, len: 2 }
            Token { kind: Newline, len: 1 }
            Token { kind: Minus, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: And, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: Or, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: Plus, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: Star, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: Slash, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: Caret, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: Percent, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: StarStar, len: 2 }
            Token { kind: Newline, len: 1 }
            Token { kind: SlashSlash, len: 2 }
            Token { kind: Newline, len: 1 }
            Token { kind: LtLt, len: 2 }
            Token { kind: Newline, len: 1 }
            Token { kind: GtGt, len: 2 }
            Token { kind: Newline, len: 1 }
            Token { kind: DotDotDot, len: 3 }
            Token { kind: Newline, len: 1 }
            Token { kind: PlusEq, len: 2 }
            Token { kind: Newline, len: 1 }
            Token { kind: MinusEq, len: 2 }
            Token { kind: Newline, len: 1 }
            Token { kind: StarEq, len: 2 }
            Token { kind: Newline, len: 1 }
            Token { kind: SlashEq, len: 2 }
            Token { kind: Newline, len: 1 }
            Token { kind: PercentEq, len: 2 }
            Token { kind: Newline, len: 1 }
            Token { kind: AndEq, len: 2 }
            Token { kind: Newline, len: 1 }
            Token { kind: OrEq, len: 2 }
            Token { kind: Newline, len: 1 }
            Token { kind: CaretEq, len: 2 }
            Token { kind: Newline, len: 1 }
            Token { kind: StarStarEq, len: 3 }
            Token { kind: Newline, len: 1 }
            Token { kind: SlashSlashEq, len: 3 }
            Token { kind: Newline, len: 1 }
            Token { kind: LtLtEq, len: 3 }
            Token { kind: Newline, len: 1 }
            Token { kind: GtGtEq, len: 3 }
            Token { kind: Newline, len: 1 }
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
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Int { base: Decimal, empty_int: false }, suffix_start: 4 }, len: 4 }
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Int { base: Binary, empty_int: false }, suffix_start: 5 }, len: 5 }
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Int { base: Hexadecimal, empty_int: false }, suffix_start: 5 }, len: 5 }
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Float { base: Decimal, empty_exponent: false }, suffix_start: 3 }, len: 3 }
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Float { base: Decimal, empty_exponent: false }, suffix_start: 6 }, len: 6 }
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Int { base: Octal, empty_int: false }, suffix_start: 4 }, len: 4 }
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Int { base: Octal, empty_int: false }, suffix_start: 4 }, len: 4 }
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Int { base: Decimal, empty_int: false }, suffix_start: 1 }, len: 3 }
            Token { kind: Newline, len: 1 }
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
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Str { terminated: true, triple_quoted: false }, suffix_start: 3 }, len: 3 }
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Str { terminated: true, triple_quoted: false }, suffix_start: 3 }, len: 3 }
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Str { terminated: true, triple_quoted: true }, suffix_start: 7 }, len: 7 }
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Str { terminated: true, triple_quoted: true }, suffix_start: 7 }, len: 7 }
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Str { terminated: true, triple_quoted: false }, suffix_start: 4 }, len: 4 }
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Str { terminated: true, triple_quoted: false }, suffix_start: 4 }, len: 4 }
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Str { terminated: true, triple_quoted: true }, suffix_start: 8 }, len: 8 }
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Str { terminated: true, triple_quoted: true }, suffix_start: 8 }, len: 8 }
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Str { terminated: true, triple_quoted: false }, suffix_start: 4 }, len: 4 }
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Str { terminated: true, triple_quoted: false }, suffix_start: 4 }, len: 4 }
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Str { terminated: true, triple_quoted: true }, suffix_start: 8 }, len: 8 }
            Token { kind: Newline, len: 1 }
            Token { kind: Literal { kind: Str { terminated: true, triple_quoted: true }, suffix_start: 8 }, len: 8 }
            Token { kind: Newline, len: 1 }
        "#]],
    )
}

#[test]
fn identifier() {
    check_lexing(
        r####"
a
abc
$schema
"####,
        expect![[r#"
            Token { kind: Newline, len: 1 }
            Token { kind: Ident, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: Ident, len: 3 }
            Token { kind: Newline, len: 1 }
            Token { kind: Ident, len: 7 }
            Token { kind: Newline, len: 1 }
        "#]],
    )
}

#[test]
fn line_continue() {
    check_lexing(
        r####"
\
\
"####,
        expect![[r#"
            Token { kind: Newline, len: 1 }
            Token { kind: LineContinue, len: 2 }
            Token { kind: LineContinue, len: 2 }
        "#]],
    )
}

#[test]
fn newline_r_n() {
    check_lexing(
        "\r\n",
        expect![[r#"
            Token { kind: Whitespace, len: 1 }
            Token { kind: Newline, len: 1 }
        "#]],
    )
}

#[test]
fn newline_r_n_r_n() {
    check_lexing(
        "\r\n\r\n",
        expect![[r#"
            Token { kind: Whitespace, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Newline, len: 1 }
        "#]],
    )
}

#[test]
fn newline_r() {
    check_lexing(
        "\r",
        expect![[r#"
            Token { kind: Whitespace, len: 1 }
        "#]],
    )
}

#[test]
fn newline_r_n_n() {
    check_lexing(
        "\r\n\n",
        expect![[r#"
            Token { kind: Whitespace, len: 1 }
            Token { kind: Newline, len: 1 }
            Token { kind: Newline, len: 1 }
        "#]],
    )
}
