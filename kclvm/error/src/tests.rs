use crate::*;

#[test]
fn test_bug_macro() {
    let result = std::panic::catch_unwind(|| {
        bug!();
    });
    assert!(result.is_err());
    let result = std::panic::catch_unwind(|| {
        bug!("an error msg");
    });
    assert!(result.is_err());
    let result = std::panic::catch_unwind(|| {
        bug!("an error msg with string format {}", "msg");
    });
    assert!(result.is_err());
}

#[test]
fn test_handler_parse_error() {
    let result = std::panic::catch_unwind(|| {
        let mut handler = Handler::default();
        handler.add_parse_error(
            ParseError::unexpected_token(&["+", "-", "*", "/"], "//"),
            Position::dummy_pos(),
        );
        handler.abort_if_errors();
    });
    assert!(result.is_err());
}
