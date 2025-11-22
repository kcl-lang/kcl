use regex::Regex;

#[inline]
pub fn is_private_field(name: &str) -> bool {
    name.starts_with('_')
}

#[inline]
pub fn is_valid_kcl_name(name: &str) -> bool {
    let re = Regex::new(r#"^[a-zA-Z_][a-zA-Z0-9_]*$"#).unwrap();
    re.is_match(name)
}
