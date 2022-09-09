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

    use std::{fs, path::PathBuf, sync::Arc};

    use crate::{
        components::CodeSnippet,
        diagnostic::{components::Label, style::DiagnosticStyle, Component},
        Diagnostic,
    };
    use compiler_base_span::{span::new_byte_pos, FilePathMapping, SourceMap, SpanData};
    use rustc_errors::styled_buffer::{StyledBuffer, StyledString};

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
        StyledString::<DiagnosticStyle>::new(
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

        StyledString::<DiagnosticStyle>::new("This is a string with no style".to_string(), None)
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

    #[test]
    fn test_code_span() {
        let filename = fs::canonicalize(&PathBuf::from("./src/diagnostic/test_datas/code_snippet"))
            .unwrap()
            .display()
            .to_string();

        let src = std::fs::read_to_string(filename.clone()).unwrap();
        let sm = SourceMap::new(FilePathMapping::empty());
        sm.new_source_file(PathBuf::from(filename.clone()).into(), src);

        let code_span = SpanData {
            lo: new_byte_pos(23),
            hi: new_byte_pos(25),
        }
        .span();

        let code_span = CodeSnippet::new(code_span, Arc::new(sm));
        let mut diag = Diagnostic::new();
        diag.append_component(Box::new(code_span));

        let mut sb = StyledBuffer::<DiagnosticStyle>::new();
        let mut errs = vec![];
        diag.format(&mut sb, &mut errs);

        let result = sb.render();
        assert_eq!(errs.len(), 0);

        assert_eq!(errs.len(), 0);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(0).unwrap().len(), 6);
        let expected_path = format!("---> File: {}:2:3: 2:5", filename);
        assert_eq!(result.get(0).unwrap().get(0).unwrap().text, expected_path);
        assert_eq!(result.get(0).unwrap().get(1).unwrap().text, "\n  ");
        assert_eq!(result.get(0).unwrap().get(2).unwrap().text, "1");
        assert_eq!(
            result.get(0).unwrap().get(3).unwrap().text,
            "|Line 2 Code Snippet.\n   |  "
        );
        assert_eq!(result.get(0).unwrap().get(4).unwrap().text, "^^");
        assert_eq!(result.get(0).unwrap().get(5).unwrap().text, "\n");
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
        let msg_in_line = template_loader.get_msg_to_str(index, sub_index, args);
        assert_eq!(msg_in_line.unwrap(), expected_msg);
    }
}
