mod test_diagnostic {
    use crate::diagnostic::{components::Label, style::DiagnosticStyle, Component, Diagnostic};
    use rustc_errors::styled_buffer::StyledBuffer;

    #[test]
    fn test_diagnostic_with_label() {
        let mut diagnostic = Diagnostic::new();

        let err_label = Box::new(Label::Error("E3033".to_string()));
        diagnostic.append_component(err_label);

        let msg = Box::new(": this is an error!".to_string());
        diagnostic.append_component(msg);

        let mut sb = StyledBuffer::<DiagnosticStyle>::new();

        let mut errs = vec![];
        diagnostic.format(&mut sb, &mut errs);
        let result = sb.render();

        assert_eq!(result.len(), 1);
        assert_eq!(result.get(0).unwrap().len(), 3);
        assert_eq!(result.get(0).unwrap().get(0).unwrap().text, "error");
        assert_eq!(result.get(0).unwrap().get(1).unwrap().text, "[E3033]");
        assert_eq!(
            result.get(0).unwrap().get(2).unwrap().text,
            ": this is an error!"
        );

        assert_eq!(
            result.get(0).unwrap().get(0).unwrap().style,
            Some(DiagnosticStyle::NeedFix)
        );
        assert_eq!(
            result.get(0).unwrap().get(1).unwrap().style,
            Some(DiagnosticStyle::Helpful)
        );
        assert_eq!(result.get(0).unwrap().get(2).unwrap().style, None);
    }
}

mod test_components {

    use crate::{
        components::StringWithStyle,
        diagnostic::{components::Label, style::DiagnosticStyle, Component},
    };
    use rustc_errors::styled_buffer::StyledBuffer;

    #[test]
    fn test_label() {
        let mut sb = StyledBuffer::<DiagnosticStyle>::new();
        let mut errs = vec![];
        Label::Error("E3030".to_string()).format(&mut sb, &mut errs);
        Label::Warning("W3030".to_string()).format(&mut sb, &mut errs);
        Label::Note.format(&mut sb, &mut errs);
        Label::Help.format(&mut sb, &mut errs);
        let result = sb.render();
        assert_eq!(errs.len(), 0);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(0).unwrap().len(), 6);
        assert_eq!(result.get(0).unwrap().get(0).unwrap().text, "error");
        assert_eq!(result.get(0).unwrap().get(1).unwrap().text, "[E3030]");
        assert_eq!(result.get(0).unwrap().get(2).unwrap().text, "warning");
        assert_eq!(result.get(0).unwrap().get(3).unwrap().text, "[W3030]");
        assert_eq!(result.get(0).unwrap().get(4).unwrap().text, "note");
        assert_eq!(result.get(0).unwrap().get(5).unwrap().text, "help");
    }

    #[test]
    fn test_string() {
        let mut sb = StyledBuffer::<DiagnosticStyle>::new();
        let mut errs = vec![];
        "this is a component string"
            .to_string()
            .format(&mut sb, &mut errs);
        let result = sb.render();
        assert_eq!(errs.len(), 0);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(0).unwrap().len(), 1);
        assert_eq!(
            result.get(0).unwrap().get(0).unwrap().text,
            "this is a component string"
        );
        assert_eq!(result.get(0).unwrap().get(0).unwrap().style, None);
    }

    #[test]
    fn test_string_with_style() {
        let mut sb = StyledBuffer::<DiagnosticStyle>::new();
        let mut errs = vec![];
        StringWithStyle::new_with_style(
            "This is a string with NeedFix style".to_string(),
            Some(DiagnosticStyle::NeedFix),
        )
        .format(&mut sb, &mut errs);
        let result = sb.render();
        assert_eq!(errs.len(), 0);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(0).unwrap().len(), 1);
        assert_eq!(
            result.get(0).unwrap().get(0).unwrap().text,
            "This is a string with NeedFix style"
        );
        assert_eq!(
            result.get(0).unwrap().get(0).unwrap().style.unwrap(),
            DiagnosticStyle::NeedFix
        );

        StringWithStyle::new_with_no_style("This is a string with no style".to_string())
            .format(&mut sb, &mut errs);
        let result = sb.render();
        assert_eq!(errs.len(), 0);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(0).unwrap().len(), 2);
        assert_eq!(
            result.get(0).unwrap().get(1).unwrap().text,
            "This is a string with no style"
        );
        assert_eq!(result.get(0).unwrap().get(1).unwrap().style, None);
    }
}

mod test_error_message {
    use crate::{diagnostic::diagnostic_message::TemplateLoader, diagnostic_handler::MessageArgs};

    #[test]
    fn test_template_message() {
        let template_dir = "./src/diagnostic/locales/en-US";
        let template_loader = TemplateLoader::new_with_template_dir(template_dir).unwrap();

        let mut args = MessageArgs::new();
        check_template_msg(
            "invalid-syntax",
            None,
            &args,
            "Invalid syntax",
            &template_loader,
        );

        args.set("expected_items", "I am an expected item");
        check_template_msg(
            "invalid-syntax",
            Some("expected"),
            &args,
            "Expected one of `\u{2068}I am an expected item\u{2069}`",
            &template_loader,
        );

        args.set("expected_items", "I am an expected item");
        check_template_msg(
            "invalid-syntax-1",
            Some("expected_1"),
            &args,
            "Expected one of `\u{2068}I am an expected item\u{2069}` 1",
            &template_loader,
        );
    }

    fn check_template_msg(
        index: &str,
        sub_index: Option<&str>,
        args: &MessageArgs,
        expected_msg: &str,
        template_loader: &TemplateLoader,
    ) {
        let msg_in_line = template_loader.get_msg_to_str(index, sub_index, &args);
        assert_eq!(msg_in_line.unwrap(), expected_msg);
    }
}
