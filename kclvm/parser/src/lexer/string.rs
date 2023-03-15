//! This module mainly provides KCL literals into forms that can be
//! represented by Rust literals.
//!
//! Reference:
//! - https://github.com/RustPython/RustPython/blob/main/parser/src/lexer.rs
//! - https://docs.python.org/3/library/string.html
use std::str::Chars;

/// Eval a string content a Rust string.
/// todo: use in place algorithm through pointers.
pub(crate) fn str_content_eval(
    string_content: &str,
    quote_char: char,
    triple_quoted: bool,
    is_bytes: bool,
    is_raw: bool,
) -> Option<String> {
    let mut chars: std::iter::Peekable<Chars> = string_content.chars().peekable();
    let mut string_content = String::new();
    loop {
        match chars.next() {
            Some('\\') => {
                let next_char = chars.next();
                if next_char == Some(quote_char) && !is_raw {
                    string_content.push(quote_char);
                } else if is_raw {
                    string_content.push('\\');
                    if let Some(c) = next_char {
                        string_content.push(c)
                    } else {
                        return None;
                    }
                } else {
                    match next_char {
                        Some('\\') => {
                            string_content.push('\\');
                        }
                        Some('\'') => string_content.push('\''),
                        Some('\"') => string_content.push('\"'),
                        Some('\n') => {
                            // Ignore Unix EOL character
                        }
                        Some('a') => string_content.push('\x07'),
                        Some('b') => string_content.push('\x08'),
                        Some('f') => string_content.push('\x0c'),
                        Some('n') => {
                            string_content.push('\n');
                        }
                        Some('r') => string_content.push('\r'),
                        Some('t') => {
                            string_content.push('\t');
                        }
                        Some('v') => string_content.push('\x0b'),
                        Some(o @ '0'..='7') => string_content.push(parse_octet(o, &mut chars)),
                        Some('x') => string_content.push(unicode_literal(2, &mut chars)?),
                        Some('u') if !is_bytes => {
                            string_content.push(unicode_literal(4, &mut chars)?)
                        }
                        Some('U') if !is_bytes => {
                            string_content.push(unicode_literal(8, &mut chars)?)
                        }
                        Some('N') if !is_bytes => {
                            string_content.push(parse_unicode_name(&mut chars)?)
                        }
                        Some(c) => {
                            string_content.push('\\');
                            string_content.push(c);
                        }
                        None => return None,
                    }
                }
            }
            Some(c) => {
                if (c == '\n' && !triple_quoted) || (is_bytes && !c.is_ascii()) {
                    return None;
                }
                string_content.push(c);
            }
            None => break,
        }
    }
    Some(string_content)
}

/// Parse unicode literal.
fn unicode_literal(literal_number: usize, chars: &mut std::iter::Peekable<Chars>) -> Option<char> {
    let mut p: u32 = 0u32;
    for i in 1..=literal_number {
        match chars.next() {
            Some(c) => match c.to_digit(16) {
                Some(d) => p += d << ((literal_number - i) * 4),
                None => return None,
            },
            None => return None,
        }
    }
    match p {
        0xD800..=0xDFFF => Some(std::char::REPLACEMENT_CHARACTER),
        _ => std::char::from_u32(p),
    }
}

fn parse_octet(first: char, chars: &mut std::iter::Peekable<Chars>) -> char {
    let mut octet_content = String::new();
    octet_content.push(first);
    while octet_content.len() < 3 {
        let next_char = chars.next();
        match next_char {
            Some(o @ '0'..='7') => octet_content.push(o),
            _ => break,
        }
    }
    let value = u32::from_str_radix(&octet_content, 8).unwrap();
    char::from_u32(value).unwrap()
}

fn parse_unicode_name(chars: &mut std::iter::Peekable<Chars>) -> Option<char> {
    match chars.next() {
        Some('{') => {}
        _ => return None,
    }
    let mut name = String::new();
    loop {
        match chars.next() {
            Some('}') => break,
            Some(c) => name.push(c),
            None => return None,
        }
    }
    unicode_names2::character(&name)
}
