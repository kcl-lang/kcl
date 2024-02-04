use anyhow::Result;

/// Parse attribute path which returns either a vector of strings or an error. e.g.
/// `a.b.c`, `a['b'].c`, `a["b"].c`, `a.['b'].c` and `a.["b"].c` both return `["a", "b", "c"]`
pub fn parse_attribute_path(path: &str) -> Result<Vec<String>> {
    let mut parts: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut chars = path.chars().peekable();
    let mut in_brackets = false;

    while let Some(ch) = chars.next() {
        if in_brackets {
            if ch == '"' || ch == '\'' {
                // Expecting the closing quote, skip if found
                if chars.peek() == Some(&']') {
                    chars.next(); // Consume the closing bracket
                    in_brackets = false;
                    continue;
                }
                return Err(anyhow::anyhow!("Expected closing bracket"));
            } else {
                current.push(ch);
            }
        } else {
            match ch {
                '.' => {
                    if !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                }
                '[' => {
                    if !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                    in_brackets = true;
                    // Skip the opening quote
                    if let Some(next_char) = chars.next() {
                        if next_char != '"' && next_char != '\'' {
                            return Err(anyhow::anyhow!("Expected opening quote after '['"));
                        }
                    }
                }
                ']' => {
                    return Err(anyhow::anyhow!("Unmatched closing bracket"));
                }
                _ => {
                    current.push(ch);
                }
            }
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    Ok(parts)
}
