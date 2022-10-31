mod test_session {
    use std::{path::PathBuf, sync::Arc};

    use crate::{Session, SessionDiagnostic};
    use anyhow::Result;
    use compiler_base_error::{
        components::{CodeSnippet, Label},
        Diagnostic, DiagnosticStyle,
    };
    use compiler_base_span::{span::new_byte_pos, Span};

    const CARGO_ROOT: &str = env!("CARGO_MANIFEST_DIR");
    #[test]
    fn test_new_session_with_filename() {
        let mut cargo_file_path = PathBuf::from(CARGO_ROOT);
        cargo_file_path.push("src/test_datas/code_snippet");
        let abs_path = cargo_file_path.to_str().unwrap();
        match Session::new_with_file_and_code(abs_path, None) {
            Ok(_) => {}
            Err(_) => {
                panic!("Unreachable")
            }
        }
    }

    #[test]
    fn test_new_session_with_filename_and_src() {
        let mut cargo_file_path = PathBuf::from(CARGO_ROOT);
        cargo_file_path.push("src/test_datas/code_snippet");
        let abs_path = cargo_file_path.to_str().unwrap();
        match Session::new_with_file_and_code(abs_path, Some("Hello World")) {
            Ok(_) => {}
            Err(_) => {
                panic!("Unreachable")
            }
        }
    }

    #[test]
    fn test_new_session_with_filename_invalid() {
        let mut cargo_file_path = PathBuf::from(CARGO_ROOT);
        cargo_file_path.push("src/test_datas/no_exists");
        let abs_path = cargo_file_path.to_str().unwrap();
        match Session::new_with_file_and_code(abs_path, None) {
            Ok(_) => {
                panic!("Unreachable")
            }
            Err(err) => {
                assert_eq!(err.to_string(), "Failed to load source file")
            }
        }
    }

    // 1. Create your own error type.
    struct MyError;

    // 2. Implement trait `SessionDiagnostic` manually.
    impl SessionDiagnostic for MyError {
        fn into_diagnostic(self, _: &Session) -> Result<Diagnostic<DiagnosticStyle>> {
            let mut diag = Diagnostic::<DiagnosticStyle>::new();
            // Label Component
            let label_component = Box::new(Label::Error("error".to_string()));
            diag.append_component(label_component);
            Ok(diag)
        }
    }

    #[test]
    fn test_session_emit_err() {
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let result = std::panic::catch_unwind(|| {
            // 3. Create a Session.
            let sess = Session::new_with_src_code("test code").unwrap();
            // 4. Add the error diagnostic.
            sess.add_err(MyError {}).unwrap();
            // 5. Emit the error diagnostic.
            sess.emit_stashed_diagnostics_and_abort().unwrap();
        });
        assert!(result.is_err());
        std::panic::set_hook(prev_hook);
    }

    // 1. Create your own error type.
    struct CodeSnippetError {
        span: Span,
    }

    // 2. Implement trait `SessionDiagnostic` manually.
    impl SessionDiagnostic for CodeSnippetError {
        fn into_diagnostic(self, sess: &Session) -> Result<Diagnostic<DiagnosticStyle>> {
            let mut diag = Diagnostic::<DiagnosticStyle>::new();
            // Label Component
            let label_component = Box::new(Label::Error("error".to_string()));
            diag.append_component(label_component);

            let msg_component = Box::new(": This is a code snippet error.".to_string());
            diag.append_component(msg_component);

            let code_snippet_component =
                Box::new(CodeSnippet::new(self.span, Arc::clone(&sess.sm)));
            diag.append_component(code_snippet_component);

            let msg_component_1 = Box::new("This is the bad code snippet.".to_string());
            diag.append_component(msg_component_1);

            Ok(diag)
        }
    }

    #[test]
    fn test_session_emit_code_snippet_err() {
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));

        let mut cargo_file_path = PathBuf::from(CARGO_ROOT);
        cargo_file_path.push("src/test_datas/code_snippet");
        let abs_path = cargo_file_path.to_str().unwrap();

        let result = std::panic::catch_unwind(|| {
            // Create a Session with no src code.
            let sess = Session::new_with_file_and_code(abs_path, None).unwrap();
            // Add the error diagnostic.
            sess.add_err(CodeSnippetError {
                span: Span::new(new_byte_pos(0), new_byte_pos(8)),
            })
            .unwrap();
            // Emit the error diagnostic.
            sess.emit_stashed_diagnostics_and_abort().unwrap();
        });
        assert!(result.is_err());

        let result_with_src = std::panic::catch_unwind(|| {
            // Create a Session with src code.
            let sess_with_src =
                Session::new_with_file_and_code(abs_path, Some("This is session with src code ."))
                    .unwrap();
            // Add the error diagnostic.
            sess_with_src
                .add_err(CodeSnippetError {
                    span: Span::new(new_byte_pos(0), new_byte_pos(8)),
                })
                .unwrap();
            // Emit the error diagnostic.
            sess_with_src.emit_stashed_diagnostics_and_abort().unwrap();
        });
        assert!(result_with_src.is_err());

        std::panic::set_hook(prev_hook);
    }

    #[test]
    fn test_emit_stashed_diagnostics() {
        let sess = Session::new_with_src_code("test code").unwrap();
        sess.add_err(MyError {}).unwrap();
        sess.emit_stashed_diagnostics().unwrap();
    }

    #[test]
    fn test_add_err() {
        let sess = Session::new_with_src_code("test code").unwrap();
        assert_eq!(sess.diagnostics_count().unwrap(), 0);
        sess.add_err(MyError {}).unwrap();
        assert_eq!(sess.diagnostics_count().unwrap(), 1);
    }

    struct MyWarning;

    impl SessionDiagnostic for MyWarning {
        fn into_diagnostic(self, _: &Session) -> Result<Diagnostic<DiagnosticStyle>> {
            let mut diag = Diagnostic::<DiagnosticStyle>::new();
            // Label Component
            let label_component = Box::new(Label::Warning("warning".to_string()));
            diag.append_component(label_component);
            Ok(diag)
        }
    }

    #[test]
    fn test_add_warn() {
        let sess = Session::new_with_src_code("test code").unwrap();
        assert_eq!(sess.diagnostics_count().unwrap(), 0);
        sess.add_warn(MyWarning {}).unwrap();
        assert_eq!(sess.diagnostics_count().unwrap(), 1);
    }
}
