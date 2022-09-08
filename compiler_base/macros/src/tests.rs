mod test_bug {
    use crate::bug;

    #[test]
    fn test_bug_macro() {
        std::panic::set_hook(Box::new(|_| {}));
        let result = std::panic::catch_unwind(|| {
            bug!();
        });
        assert!(result.is_err());
        let result = std::panic::catch_unwind(|| {
            bug!("an error msg");
        });
        assert!(result.is_err());
        match result {
            Ok(_) => panic!("test bug!() failed"),
            Err(panic_err) => {
                let err_message = if let Some(s) = panic_err.downcast_ref::<String>() {
                    (*s).clone()
                } else {
                    panic!("test bug!() failed")
                };
                assert_eq!(
                    err_message,
                    "Internal error, please report a bug to us. The error message is: an error msg"
                );
            }
        }

        let result = std::panic::catch_unwind(|| {
            bug!("an error msg with string format {}", "msg");
        });
        assert!(result.is_err());
        match result {
            Ok(_) => panic!("test bug!() failed"),
            Err(panic_err) => {
                let err_message = if let Some(s) = panic_err.downcast_ref::<String>() {
                    (*s).clone()
                } else {
                    panic!("test bug!() failed")
                };
                assert_eq!(err_message, "Internal error, please report a bug to us. The error message is: an error msg with string format msg");
            }
        }
    }
}
