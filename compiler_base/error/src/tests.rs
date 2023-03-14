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
    use crate::{Component, Diagnostic, DiagnosticStyle, Emitter, EmitterWriter};

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

        let mut emitter = EmitterWriter::default();
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

mod test_emitter {
    use crate::{
        components::Label, diagnostic_handler::DiagnosticHandler,
        emit_diagnostic_to_uncolored_text, emitter::Destination, Diagnostic, Emitter,
        EmitterWriter,
    };
    use std::io::{self, Write};
    use termcolor::Ansi;

    struct MyWriter {
        content: String,
    }

    impl Write for MyWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if let Ok(s) = std::str::from_utf8(buf) {
                self.content.push_str(s)
            } else {
                self.content = "Nothing".to_string();
            }
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    unsafe impl Send for MyWriter {}

    #[test]
    fn test_emit_to_raw() {
        let mut writer = MyWriter {
            content: String::new(),
        };
        {
            let mut emitter =
                EmitterWriter::new_with_writer(Destination::ColoredRaw(Ansi::new(&mut writer)));
            let mut diag = Diagnostic::new();
            diag.append_component(Box::new(Label::Note));
            emitter.emit_diagnostic(&diag).unwrap();
        }

        assert_eq!(
            writer.content,
            "\u{1b}[0m\u{1b}[1m\u{1b}[38;5;14mnote\u{1b}[0m"
        );
        writer.content = String::new();
        {
            let mut emitter =
                EmitterWriter::new_with_writer(Destination::UnColoredRaw(&mut writer));
            let mut diag = Diagnostic::new();
            diag.append_component(Box::new(Label::Note));
            emitter.emit_diagnostic(&diag).unwrap();
        }

        assert_eq!(writer.content, "note");
    }

    #[test]
    fn test_emit_diag_to_uncolored_text() {
        let mut diag = Diagnostic::new();
        diag.append_component(Box::new(Label::Note));
        assert_eq!(emit_diagnostic_to_uncolored_text(&diag).unwrap(), "note");
    }

    struct EmitResultText {
        pub text_res: String,
    }

    impl Write for EmitResultText {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if let Ok(s) = std::str::from_utf8(buf) {
                self.text_res.push_str(s)
            } else {
                self.text_res = String::new();
            }
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    unsafe impl Send for EmitResultText {}

    #[test]
    fn test_emit_diag_to_uncolored_text_many_times() {
        let mut emit_res = EmitResultText {
            text_res: String::new(),
        };
        {
            let mut emit_writter =
                EmitterWriter::new_with_writer(Destination::UnColoredRaw(&mut emit_res));
            let mut diag = Diagnostic::new();
            diag.append_component(Box::new(Label::Note));
            emit_writter.emit_diagnostic(&diag).unwrap();
            emit_writter.emit_diagnostic(&diag).unwrap();
            emit_writter.emit_diagnostic(&diag).unwrap();
        }
        assert_eq!(emit_res.text_res, "notenotenote");
    }
}
