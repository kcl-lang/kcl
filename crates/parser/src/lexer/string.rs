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
        let next_char = chars.peek();
        match next_char {
            Some(o @ '0'..='7') => {
                octet_content.push(*o);
                chars.next();
            }
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

/// Dedent a multi-line string by stripping the common leading whitespace
/// from all non-empty lines, including the closing quote's line.
/// Additionally, if the first line is completely empty (i.e. a newline immediately
/// following the opening quotes), it is ignored and removed.
pub(crate) fn dedent_string(s: &str) -> String {
    let lines = s.split('\n');
    let mut min_indent = usize::MAX;
    
    let lines_vec: Vec<&str> = lines.collect();
    
    // Find the minimum indent of all non-empty lines, EXCEPT the first line
    // (if it's empty, it won't be counted anyway because we check !trim().is_empty()).
    // However, if the first line has text right after the opening quotes (e.g. `"""foo`),
    // it technically has 0 indentation.
    for line in lines_vec.iter() {
        if !line.trim().is_empty() {
            let indent = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
            min_indent = min_indent.min(indent);
        }
    }
    
    // Check if the last line is just whitespace (e.g., closing quotes indented)
    // We want to use its indentation as a baseline too.
    if let Some(last) = lines_vec.last() {
        if last.trim().is_empty() && !last.is_empty() {
            let indent = last.chars().take_while(|c| *c == ' ' || *c == '\t').count();
            min_indent = min_indent.min(indent);
        }
    }
    
    if min_indent == usize::MAX {
        min_indent = 0;
    }
    
    // Determine whether to skip the first empty line
    let skip_first = lines_vec.len() > 1 && lines_vec[0].is_empty();
    
    let mut result = String::new();
    for (i, line) in lines_vec.iter().enumerate() {
        if i == 0 && skip_first {
            continue;
        }
        
        if i > 0 && !(i == 1 && skip_first) {
            result.push('\n');
        }
        
        let chars_to_skip = line.chars().take_while(|c| *c == ' ' || *c == '\t').count().min(min_indent);
        
        // Use byte indices to split safely, assuming ASCII space/tabs
        let mut byte_offset = 0;
        let mut char_count = 0;
        for (j, _c) in line.char_indices() {
            if char_count == chars_to_skip {
                byte_offset = j;
                break;
            }
            char_count += 1;
        }
        if char_count < chars_to_skip {
            byte_offset = line.len(); // fallback if string is too short (shouldn't happen due to min)
        } else if char_count == chars_to_skip && chars_to_skip == line.chars().count() {
            byte_offset = line.len(); // if we skip the entire string
        }
        
        let (_, dedented) = line.split_at(byte_offset);
        result.push_str(dedented);
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dedent_string() {
        let s = "
    foo
    bar
    ";
        assert_eq!(dedent_string(s), "foo\nbar\n");

        let s = "
        import os
        
        print os
        ";
        assert_eq!(dedent_string(s), "import os\n\nprint os\n");

        // No leading newline
        let s = "    foo\n    bar";
        assert_eq!(dedent_string(s), "foo\nbar");

        // Different indent levels
        let s = "
    foo
      bar
    ";
        assert_eq!(dedent_string(s), "foo\n  bar\n");
        
        // Only one line with spaces
        let s = "  foo  ";
        assert_eq!(dedent_string(s), "foo  ");
    }
}
