#[inline]
pub fn is_private_field(name: &str) -> bool {
    name.starts_with('_')
}
