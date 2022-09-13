mod test_diagnostic_handler {
    use std::panic;

    use crate::{
        diagnostic_handler::{DiagnosticHandler, MessageArgs},
        Diagnostic, DiagnosticStyle,
    };

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
        if diag_handler_invalid.is_ok() {
            panic!("`diag_handler_invalid` should be Err(...)")
        }
    }

    #[test]
    fn test_diagnostic_handler_add_diagnostic() {
        let diag_1 = Diagnostic::<DiagnosticStyle>::new();
        let diag_handler =
            DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
        assert_eq!(diag_handler.diagnostics_count().unwrap(), 0);

        diag_handler.add_err_diagnostic(diag_1).unwrap();
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
        assert!(!diag_handler.has_errors().unwrap());
        diag_handler
            .add_err_diagnostic(Diagnostic::<DiagnosticStyle>::new())
            .unwrap();
        assert!(diag_handler.has_errors().unwrap());

        // test has_warns()
        assert!(!diag_handler.has_warns().unwrap());
        diag_handler
            .add_warn_diagnostic(Diagnostic::<DiagnosticStyle>::new())
            .unwrap();
        assert!(diag_handler.has_warns().unwrap());
    }

    #[test]
    fn test_abort_if_errors() {
        let diag_handler =
            DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
        diag_handler.abort_if_errors().unwrap();
        diag_handler
            .add_warn_diagnostic(Diagnostic::<DiagnosticStyle>::new())
            .unwrap();
        diag_handler.abort_if_errors().unwrap();
        diag_handler
            .add_err_diagnostic(Diagnostic::<DiagnosticStyle>::new())
            .unwrap();

        let result = panic::catch_unwind(|| {
            diag_handler.abort_if_errors().unwrap();
        });
        assert!(result.is_err());
    }
}

mod test_errors {
    use rustc_errors::styled_buffer::StyledBuffer;

    use crate::errors::{ComponentError, ComponentFormatError};
    use crate::{Component, Diagnostic, DiagnosticStyle, Emitter, TerminalEmitter};

    // Component to generate errors.
    struct ComponentGenError;
    impl Component<DiagnosticStyle> for ComponentGenError {
        fn format(
            &self,
            _: &mut StyledBuffer<DiagnosticStyle>,
            errs: &mut Vec<ComponentFormatError>,
        ) {
            errs.push(ComponentFormatError::new(
                "ComponentGenError",
                "This is an error for testing",
            ));
        }
    }

    #[test]
    fn test_component_format_error() {
        let cge = ComponentGenError {};
        let mut diagnostic = Diagnostic::<DiagnosticStyle>::new();
        diagnostic.append_component(Box::new(cge));

        let mut emitter = TerminalEmitter::default();
        match emitter.emit_diagnostic(&diagnostic) {
            Ok(_) => {
                panic!("`emit_diagnostic` shoule be failed.")
            }
            Err(err) => {
                match err.downcast_ref::<ComponentError>() {
                    Some(ce) => {
                        let err_msg = format!("{:?}", ce);
                        assert_eq!(err_msg, "ComponentFormatErrors([ComponentFormatError { name: \"ComponentGenError\", message: \"This is an error for testing\" }])")
                    }
                    None => {
                        panic!("Error Type Error")
                    }
                };
            }
        };
    }
}
