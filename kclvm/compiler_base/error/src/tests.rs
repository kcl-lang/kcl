use diagnostic::{emitter::Emitter, emitter::EmitterWriter, Position};
use diagnostic::DiagnosticBuilder;

use crate::ThisIsAnErr;

fn get_code_position() -> Position {
    let mut pos = Position::default();
    pos.filename =
        "/Users/shijun/Workspace/kusion/KCLVM_ERROR_SYS/KCLVM/test/grammar/schema/simple/main.k"
            .to_string();
    pos.line = 2;
    pos.column = Some(6);
    pos
}

#[test]
fn test_this_is_an_error() {
    let err = ThisIsAnErr {
        loc: get_code_position(),
    };
    let mut emitter = EmitterWriter::default();
    emitter.emit_err(err);
}
