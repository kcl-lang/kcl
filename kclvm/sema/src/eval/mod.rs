use std::str::Chars;

/// Eval a string starts with a quote ' or " to a Rust string.
pub fn str_literal_eval(string_lit: &str, is_bytes: bool, is_raw: bool) -> Option<String> {
    let mut chars: std::iter::Peekable<Chars> = string_lit.chars().peekable();

    let quote_char = match chars.next() {
        Some(c) => c,
        None => return None,
    };
    let mut string_content = String::new();
    // If the next two characters are also the quote character, then we have a triple-quoted
    // string; consume those two characters and ensure that we require a triple-quote to close
    let triple_quoted = if chars.peek() == Some(&quote_char) && chars.next() == Some(quote_char) {
        chars.next();
        true
    } else {
        false
    };
    loop {
        let chr = chars.next();
        match chr {
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
                if c == quote_char {
                    if triple_quoted {
                        // Look ahead at the next two characters; if we have two more
                        // quote_chars, it's the end of the string; consume the remaining
                        // closing quotes and break the loop
                        if chars.peek() == Some(&quote_char) && chars.nth(1) == Some(quote_char) {
                            chars.next();
                            chars.next();
                            break;
                        }
                        string_content.push(c);
                    } else {
                        break;
                    }
                } else {
                    if (c == '\n' && !triple_quoted) || (is_bytes && !c.is_ascii()) {
                        return None;
                    }
                    string_content.push(c);
                }
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
        if let Some('0'..='7') = next_char {
            octet_content.push(next_char.unwrap())
        } else {
            break;
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
