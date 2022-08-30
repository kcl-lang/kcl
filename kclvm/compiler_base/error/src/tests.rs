mod test_diagnostic_handler {
    use std::panic;

    use crate::{Diagnostic, DiagnosticHandler, DiagnosticStyle, MessageArgs};

    #[test]
    fn test_diagnostic_handler_new_with_template_dir() {
        let diag_handler =
            DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/");
        match diag_handler {
            Ok(_) => {}
            Err(_) => {
                panic!("`diag_handler` should be Ok(...)")
            }
        }

        let diag_handler_invalid = DiagnosticHandler::new_with_template_dir("./invalid_path");
        match diag_handler_invalid {
            Ok(_) => {
                panic!("`diag_handler_invalid` should be Err(...)")
            }
            Err(_) => {}
        }
    }

    #[test]
    fn test_diagnostic_handler_add_diagnostic() {
        let diag_1 = Diagnostic::<DiagnosticStyle>::new();
        let diag_handler =
            DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
        assert_eq!(diag_handler.diagnostics_count().unwrap(), 0);

        diag_handler.add_err_diagnostic(diag_1);
        assert_eq!(diag_handler.diagnostics_count().unwrap(), 1);
    }

    #[test]
    fn test_diagnostic_handler_get_diagnostic_msg() {
        let no_args = MessageArgs::new();
        let index = "invalid-syntax";
        let sub_index = None;
        let diag_handler =
            DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
        let msg_in_line_1 = diag_handler
            .get_diagnostic_msg(index, sub_index, &no_args)
            .unwrap();
        assert_eq!(msg_in_line_1, "Invalid syntax");

        let mut args = MessageArgs::new();
        args.set("expected_items", "I am an expected item");
        let sub_index = "expected";
        let msg_in_line_2 = diag_handler
            .get_diagnostic_msg(index, Some(sub_index), &args)
            .unwrap();
        assert_eq!(
            msg_in_line_2,
            "Expected one of `\u{2068}I am an expected item\u{2069}`"
        );
    }

    #[test]
    fn test_diagnostic_handler_has() {
        let diag_handler =
            DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
        // test has_errors()
        assert_eq!(diag_handler.has_errors().unwrap(), false);
        diag_handler.add_err_diagnostic(Diagnostic::<DiagnosticStyle>::new());
        assert_eq!(diag_handler.has_errors().unwrap(), true);

        // test has_warns()
        assert_eq!(diag_handler.has_warns().unwrap(), false);
        diag_handler.add_warn_diagnostic(Diagnostic::<DiagnosticStyle>::new());
        assert_eq!(diag_handler.has_warns().unwrap(), true);
    }

    #[test]
    fn test_abort_if_errors() {
        let diag_handler =
            DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
        diag_handler.abort_if_errors();
        diag_handler.add_warn_diagnostic(Diagnostic::<DiagnosticStyle>::new());
        diag_handler.abort_if_errors();
        diag_handler.add_err_diagnostic(Diagnostic::<DiagnosticStyle>::new());

        let result = panic::catch_unwind(|| {
            diag_handler.abort_if_errors();
        });
        assert!(result.is_err());
    }
}
