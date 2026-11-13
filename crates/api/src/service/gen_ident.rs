//! Identifier-casing helpers shared by every schema-to-source-code emitter
//! (`gen_rust`, `gen_python`, `gen_go`, `gen_java`, and future targets such
//! as TypeScript).
//!
//! Each target still owns its *field*-casing convention — Rust and Python
//! use `snake_case`, Go wants exported `PascalCase` struct fields, Java
//! wants `camelCase` fields — and its own keyword list and escaping style
//! (Rust's raw identifiers prefix with `r#`, Python/Java conventionally
//! suffix with `_`). Those per-target decisions stay in each `gen_*` module
//! as a thin wrapper around the primitives here.

/// Convert a KCL identifier (`snake_case`, `camelCase`, `kebab-case`, ...)
/// to `PascalCase`, suitable for a type/class/struct name. Falls back to a
/// `Kcl` prefix when the input sanitizes to nothing usable (empty, or
/// starting with a digit) so the result is always a valid identifier start.
pub(crate) fn to_pascal_case(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize = true;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            if capitalize {
                result.push(ch.to_ascii_uppercase());
            } else {
                result.push(ch);
            }
            capitalize = false;
        } else {
            capitalize = true;
        }
    }
    if result.is_empty() || result.as_bytes()[0].is_ascii_digit() {
        result.insert_str(0, "Kcl");
    }
    result
}

/// Convert a KCL identifier to `snake_case`, suitable for a Rust or Python
/// field name. Falls back to a `field_` prefix when the input sanitizes to
/// nothing usable (empty, or starting with a digit).
pub(crate) fn to_snake_case(name: &str) -> String {
    let mut chars = name.chars().peekable();
    let mut result = String::new();
    while let Some(ch) = chars.next() {
        if ch.is_ascii_uppercase() {
            if !result.is_empty()
                && !result.ends_with('_')
                && (result
                    .chars()
                    .last()
                    .is_some_and(|prev| prev.is_ascii_lowercase())
                    || chars.peek().is_some_and(|next| next.is_ascii_lowercase()))
            {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
        } else if ch.is_ascii_alphanumeric() || ch == '_' {
            result.push(ch);
        } else if !result.ends_with('_') {
            result.push('_');
        }
    }
    while result.ends_with('_') {
        result.pop();
    }
    if result.is_empty() || result.as_bytes()[0].is_ascii_digit() {
        result.insert_str(0, "field_");
    }
    result
}

/// Convert a KCL identifier to `camelCase`, suitable for a Java (or future
/// TypeScript) field name: same word-splitting as [`to_pascal_case`], with
/// the first letter decapitalized. Never starts with a digit since
/// `to_pascal_case` already guarantees that for its own output.
pub(crate) fn to_camel_case(name: &str) -> String {
    let pascal = to_pascal_case(name);
    let mut chars = pascal.chars();
    match chars.next() {
        Some(first) => first.to_ascii_lowercase().to_string() + chars.as_str(),
        None => pascal,
    }
}

/// Whether `name` collides with one of the target language's reserved
/// words, so callers know to escape it (e.g. `r#{name}` in Rust, `{name}_`
/// in Python).
pub(crate) fn is_keyword(name: &str, keywords: &[&str]) -> bool {
    keywords.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_pascal_case_capitalizes_after_separators_and_sanitizes() {
        assert_eq!(to_pascal_case("zip_code"), "ZipCode");
        assert_eq!(to_pascal_case("zipCode"), "ZipCode");
        assert_eq!(to_pascal_case("my-schema.name"), "MySchemaName");
        assert_eq!(to_pascal_case("123"), "Kcl123");
        assert_eq!(to_pascal_case(""), "Kcl");
    }

    #[test]
    fn to_snake_case_splits_on_case_boundaries_and_sanitizes() {
        assert_eq!(to_snake_case("zipCode"), "zip_code");
        // Runs of consecutive uppercase letters each get their own
        // underscore boundary rather than being treated as one acronym.
        assert_eq!(to_snake_case("ZIPCode"), "z_i_p_code");
        assert_eq!(to_snake_case("my-field.name"), "my_field_name");
        assert_eq!(to_snake_case("123"), "field_123");
        assert_eq!(to_snake_case(""), "field_");
    }

    #[test]
    fn is_keyword_checks_membership() {
        assert!(is_keyword("type", &["type", "match"]));
        assert!(!is_keyword("value", &["type", "match"]));
    }

    #[test]
    fn to_camel_case_decapitalizes_the_pascal_case_result() {
        assert_eq!(to_camel_case("zip_code"), "zipCode");
        assert_eq!(to_camel_case("ZipCode"), "zipCode");
        assert_eq!(to_camel_case("my-field.name"), "myFieldName");
        assert_eq!(to_camel_case("123"), "kcl123");
        assert_eq!(to_camel_case(""), "kcl");
    }
}
