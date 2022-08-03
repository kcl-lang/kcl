use crate::ThisIsAnErr;
use compiler_base_diagnostic::{ErrHandler, Position};

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
    let mut err_handler = ErrHandler::new();

    err_handler.emit_err(ThisIsAnErr {
        pos: get_code_position(),
    });
}
