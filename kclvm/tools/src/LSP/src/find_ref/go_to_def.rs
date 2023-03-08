use kclvm_error::Position;

/// Get the definition of an identifier.
pub fn go_to_def(pos: Position) -> Option<Position> {
    Some(pos)
}
