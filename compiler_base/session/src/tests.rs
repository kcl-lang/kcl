mod test_session {
    use crate::{Session, SessionDiagnostic};
    use anyhow::Result;
    use compiler_base_error::{components::Label, Diagnostic, DiagnosticStyle};
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
            // 4. Emit the error diagnostic.
            sess.emit_err(MyError {}).unwrap();
        });
        assert!(result.is_err());
        std::panic::set_hook(prev_hook);
    }
}
