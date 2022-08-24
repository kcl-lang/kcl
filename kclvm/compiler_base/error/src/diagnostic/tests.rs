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

        diagnostic.format(&mut sb);
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

    use crate::diagnostic::{components::Label, style::DiagnosticStyle, Component};
    use rustc_errors::styled_buffer::StyledBuffer;

    #[test]
    fn test_label() {
        let mut sb = StyledBuffer::<DiagnosticStyle>::new();
        Label::Error("E3030".to_string()).format(&mut sb);
        Label::Warning("W3030".to_string()).format(&mut sb);
        Label::Note.format(&mut sb);
        Label::Help.format(&mut sb);
        let result = sb.render();
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
        "this is a component string".to_string().format(&mut sb);
        let result = sb.render();
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(0).unwrap().len(), 1);
        assert_eq!(
            result.get(0).unwrap().get(0).unwrap().text,
            "this is a component string"
        );
        assert_eq!(result.get(0).unwrap().get(0).unwrap().style, None);
    }
}

mod test_error_message {
    use crate::diagnostic::error_message::{
        ErrorMessage, MessageArgs, MessageIndex, TemplateLoader,
    };

    #[test]
    fn test_template_message() {
        let template_path = "./src/diagnostic/locales/en-US/default.ftl";
        let no_args = MessageArgs::new();
        let msg_index = MessageIndex::from("invalid-syntax");
        let no_sub_msg_index = None;
        let template_msg = ErrorMessage::new_template_msg(msg_index, no_sub_msg_index, &no_args);
        let template_loader = TemplateLoader::new_with_template_path(template_path.to_string());
        let msg_in_line_1 = template_msg.trans_msg_to_str(Some(&template_loader));
        assert_eq!(msg_in_line_1, "Invalid syntax");

        let mut args = MessageArgs::new();
        args.set("expected_items", "I am an expected item");
        let msg_index = MessageIndex::from("invalid-syntax");
        let sub_msg_index = MessageIndex::from("expected");
        let template_msg = ErrorMessage::new_template_msg(msg_index, Some(sub_msg_index), &args);
        let template_loader = TemplateLoader::new_with_template_path(template_path.to_string());
        let msg_in_line_2 = template_msg.trans_msg_to_str(Some(&template_loader));
        assert_eq!(
            msg_in_line_2,
            "Expected one of `\u{2068}I am an expected item\u{2069}`"
        );
    }

    #[test]
    fn test_str_message() {
        let str_msg = ErrorMessage::new_str_msg("This is a str msg".to_string());
        assert_eq!(str_msg.trans_msg_to_str(None), "This is a str msg");
    }
}
