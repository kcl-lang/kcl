use insta::assert_debug_snapshot;
use kclvm::ErrType;

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
fn test_panic_info_from_diag() {
    let p_info: PanicInfo =
        Diagnostic::new_with_code(Level::Error, "Error message", Position::dummy_pos(), None)
            .into();
    assert_debug_snapshot!(p_info.to_json_string());
}

#[test]
fn test_diag_from_panic_info() {
    let mut panic_info = PanicInfo::default();

    panic_info.__kcl_PanicInfo__ = true;
    panic_info.message = "Invalid syntax".to_string();
    panic_info.err_type_code = ErrType::CompileError_TYPE as i32;

    panic_info.kcl_file = "filename".to_string();
    panic_info.kcl_line = 1;
    panic_info.kcl_col = 2;

    assert_debug_snapshot!(format!("{:?}", Diagnostic::from(panic_info)));
}
