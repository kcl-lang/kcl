mod test_pendant {
    mod test_header_pendant {
        use style::{styled_buffer::StyledBuffer, ShaderFactory, Style};

        use crate::{pendant::HeaderPendant, Pendant};

        #[test]
        fn test_header_pendnant() {
            let diag_label = "test_label".to_string();
            let diag_code = Some("E1010".to_string());
            let mut header_pendant = HeaderPendant::new(diag_label.to_string(), diag_code);
            header_pendant.set_logo("KCL".to_string());

            let shader = ShaderFactory::Diagnostic.get_shader();
            let mut sb = StyledBuffer::new();

            header_pendant.format(shader.clone(), &mut sb);

            let styled_strings = sb.render();

            assert_eq!(styled_strings.len(), 1);
            assert_eq!(styled_strings.get(0).unwrap().len(), 4);

            assert_eq!(styled_strings.get(0).unwrap().get(0).unwrap().text, "KCL");
            assert_eq!(
                styled_strings.get(0).unwrap().get(1).unwrap().text,
                "test_label"
            );
            assert_eq!(
                styled_strings.get(0).unwrap().get(2).unwrap().text,
                "[E1010]"
            );
            assert_eq!(styled_strings.get(0).unwrap().get(3).unwrap().text, ":");

            let mut sb = StyledBuffer::new();
            let header_pendant = HeaderPendant::new(diag_label.to_string(), None);
            header_pendant.format(shader, &mut sb);
            let styled_strings = sb.render();
            assert_eq!(styled_strings.len(), 1);
            assert_eq!(styled_strings.get(0).unwrap().len(), 1);

            assert_eq!(
                styled_strings.get(0).unwrap().get(0).unwrap().text,
                "test_label:"
            );
            assert_eq!(
                styled_strings.get(0).unwrap().get(0).unwrap().style,
                Style::NoStyle
            );
        }

        #[test]
        fn test_header_pendnant_style() {
            test_logo_header_pendnant_style_with_labels("error".to_string(), Style::NeedFix);
            test_logo_header_pendnant_style_with_labels(
                "warning".to_string(),
                Style::NeedAttention,
            );
            test_logo_header_pendnant_style_with_labels("help".to_string(), Style::NeedAttention);
            test_logo_header_pendnant_style_with_labels("note".to_string(), Style::NeedAttention);
        }

        fn test_logo_header_pendnant_style_with_labels(label: String, style: Style) {
            let shader = ShaderFactory::Diagnostic.get_shader();
            let mut sb = StyledBuffer::new();
            let header_pendant = HeaderPendant::new(label.to_string(), None);
            header_pendant.format(shader, &mut sb);
            let styled_strings = sb.render();
            assert_eq!(styled_strings.len(), 1);
            assert_eq!(styled_strings.get(0).unwrap().len(), 2);

            assert_eq!(
                styled_strings.get(0).unwrap().get(0).unwrap().text,
                label.to_string()
            );
            assert_eq!(styled_strings.get(0).unwrap().get(0).unwrap().style, style);
            assert_eq!(styled_strings.get(0).unwrap().get(1).unwrap().text, ":");
        }
    }

    mod test_code_ctx_pendant {
        use std::{fs, path::PathBuf};

        use style::{styled_buffer::StyledBuffer, ShaderFactory, Style};

        use crate::{pendant::CodeCtxPendant, Pendant, Position};

        fn get_code_position() -> Position {
            let mut pos = Position::default();
            pos.filename = fs::canonicalize(&PathBuf::from("./test_datas/main.k"))
                .unwrap()
                .display()
                .to_string();
            pos.line = 7;
            pos.column = Some(5);
            pos
        }

        #[test]
        fn test_code_ctx_pendant() {
            let code_pos = get_code_position();
            let code_pendant = CodeCtxPendant::new(code_pos.clone());
            let shader = ShaderFactory::Diagnostic.get_shader();
            let mut sb = StyledBuffer::new();

            code_pendant.format(shader, &mut sb);
            let styled_strings = sb.render();

            assert_eq!(styled_strings.len(), 4);

            assert_eq!(styled_strings.get(0).unwrap().len(), 1);
            assert_eq!(
                styled_strings.get(0).unwrap().get(0).unwrap().text,
                code_pos.info()
            );
            assert_eq!(
                styled_strings.get(0).unwrap().get(0).unwrap().style,
                Style::Url
            );

            assert_eq!(styled_strings.get(1).unwrap().len(), 1);
            let indent = code_pos.line.to_string().len() + 1;
            assert_eq!(
                styled_strings.get(1).unwrap().get(0).unwrap().text,
                format!("{:indent$}|", "")
            );
            assert_eq!(
                styled_strings.get(1).unwrap().get(0).unwrap().style,
                Style::NoStyle
            );

            assert_eq!(styled_strings.get(2).unwrap().len(), 2);
            assert_eq!(
                styled_strings.get(2).unwrap().get(0).unwrap().text,
                format!("{:<indent$}", &code_pos.line)
            );
            assert_eq!(
                styled_strings.get(2).unwrap().get(0).unwrap().style,
                Style::Url
            );
            assert_eq!(
                styled_strings.get(2).unwrap().get(1).unwrap().text,
                "|    name = _name"
            );
            assert_eq!(
                styled_strings.get(2).unwrap().get(1).unwrap().style,
                Style::NoStyle
            );

            assert_eq!(styled_strings.get(3).unwrap().len(), 2);
            assert_eq!(
                styled_strings.get(3).unwrap().get(0).unwrap().text,
                format!("{:<indent$}|", "")
            );
            assert_eq!(
                styled_strings.get(3).unwrap().get(0).unwrap().style,
                Style::NoStyle
            );
            let col = code_pos.column.unwrap() as usize;
            assert_eq!(
                styled_strings.get(3).unwrap().get(1).unwrap().text,
                format!("{:>col$}^ ", col),
            );
            assert_eq!(
                styled_strings.get(3).unwrap().get(1).unwrap().style,
                Style::NeedFix
            );
        }
    }

    mod test_no_pendant {
        use style::{styled_buffer::StyledBuffer, ShaderFactory, Style};

        use crate::{pendant::NoPendant, Pendant};

        #[test]
        fn test_no_pendnant() {
            let no_pendant = NoPendant::new();
            let shader = ShaderFactory::Diagnostic.get_shader();
            let mut sb = StyledBuffer::new();

            no_pendant.format(shader, &mut sb);
            let styled_strings = sb.render();
            assert_eq!(styled_strings.len(), 1);
            assert_eq!(styled_strings.get(0).unwrap().len(), 1);

            assert_eq!(
                styled_strings.get(0).unwrap().get(0).unwrap().style,
                Style::NoStyle
            );
            assert_eq!(styled_strings.get(0).unwrap().get(0).unwrap().text, "- ");
        }
    }
}

mod test_sentence {

}

mod test_position{
    use crate::Position;

    #[test]
    fn test_dummy_pos(){
        let pos = Position::dummy_pos();
        assert_eq!(pos.filename, "".to_string());
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, None);
    }

    #[test]
    fn test_is_valid(){
        let mut pos = Position::dummy_pos();
        assert_eq!(pos.is_valid(), true);
        pos.line = 0;
        assert_eq!(pos.is_valid(), false);
    }

    #[test]
    fn test_less(){
        let mut pos_greater = Position::dummy_pos();
        let mut pos_less = Position::dummy_pos();

        pos_greater.line = 0;
        assert_eq!(pos_less.less(&pos_greater), false);
        
        pos_greater.line = 1;
        pos_less.line = 0;
        assert_eq!(pos_less.less(&pos_greater), false);

        pos_greater.line = 1;
        pos_greater.filename = "greater filename".to_string();
        pos_less.line = 1;
        pos_less.filename = "less filename".to_string();
        assert_eq!(pos_less.less(&pos_greater), false);

        pos_greater.column = Some(2);
        pos_greater.filename = "filename".to_string();
        pos_less.column = Some(1);
        pos_less.filename = "filename".to_string();
        assert_eq!(pos_less.less(&pos_greater), true);

        pos_greater.column = Some(0);
        pos_less.column = Some(1);
        assert_eq!(pos_less.less(&pos_greater), false);

        pos_greater.column = None;
        pos_less.column = Some(1);
        assert_eq!(pos_less.less(&pos_greater), false);

        pos_greater.line = 2;
        pos_less.line = 1;
        assert_eq!(pos_less.less(&pos_greater), true);

        pos_greater.line = 0;
        pos_less.line = 1;
        assert_eq!(pos_less.less(&pos_greater), false);
    }

    #[test]
    fn test_less_equal(){
        let mut pos_greater = Position::dummy_pos();
        let mut pos_less = Position::dummy_pos();

        assert_eq!(pos_less.less_equal(&pos_greater), true);

        pos_greater.line = 0;
        assert_eq!(pos_less.less_equal(&pos_greater), false);
        
        pos_greater.line = 1;
        pos_less.line = 0;
        assert_eq!(pos_less.less_equal(&pos_greater), false);

        pos_greater.line = 1;
        pos_greater.filename = "greater filename".to_string();
        pos_less.line = 1;
        pos_less.filename = "less filename".to_string();
        assert_eq!(pos_less.less_equal(&pos_greater), false);

        pos_greater.column = Some(2);
        pos_greater.filename = "filename".to_string();
        pos_less.column = Some(1);
        pos_less.filename = "filename".to_string();
        assert_eq!(pos_less.less_equal(&pos_greater), true);

        pos_greater.column = Some(0);
        pos_less.column = Some(1);
        assert_eq!(pos_less.less_equal(&pos_greater), false);

        pos_greater.column = None;
        pos_less.column = Some(1);
        assert_eq!(pos_less.less_equal(&pos_greater), false);

        pos_greater.line = 2;
        pos_less.line = 1;
        assert_eq!(pos_less.less_equal(&pos_greater), true);

        pos_greater.line = 0;
        pos_less.line = 1;
        assert_eq!(pos_less.less_equal(&pos_greater), false);
        assert_eq!(pos_less.less_equal(&pos_less), true);
    }

    #[test]
    fn test_info(){
        let mut dummy_pos = Position::dummy_pos();
        assert_eq!(dummy_pos.info(), "---> File :1");
        let col = 12;
        dummy_pos.column = Some(col);
        let expected = format!("---> File :1:{}", col+1);
        assert_eq!(dummy_pos.info(), expected);
    }
}