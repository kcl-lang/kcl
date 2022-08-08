use std::{fs, path::PathBuf};

use compiler_base_style::{diagnostic_style::DiagnosticStyle, Style};
use rustc_errors::styled_buffer::StyledString;

use crate::Position;

mod test_pendant {
    mod test_header_pendant {

        use compiler_base_style::{ShaderFactory, diagnostic_style::DiagnosticStyle, Style};
        use rustc_errors::styled_buffer::StyledBuffer;

        use crate::{pendant::HeaderPendant, tests::check_styled_strings, Pendant};

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
            let expected_texts = vec![vec![
                "KCL".to_string(),
                "test_label".to_string(),
                "[E1010]".to_string(),
                ":".to_string(),
            ]];
            let expected_styles = vec![vec![
                DiagnosticStyle::Logo,
                DiagnosticStyle::Normal,
                DiagnosticStyle::Helpful,
                DiagnosticStyle::Normal,
            ]];

            check_styled_strings(
                &styled_strings,
                1,
                &vec![4],
                &expected_texts,
                &expected_styles,
            );
        }

        #[test]
        fn test_header_pendnant_style() {
            test_logo_header_pendnant_style_with_labels("error".to_string(), DiagnosticStyle::NeedFix);
            test_logo_header_pendnant_style_with_labels(
                "warning".to_string(),
                DiagnosticStyle::NeedAttention,
            );
            test_logo_header_pendnant_style_with_labels("help".to_string(), DiagnosticStyle::NeedAttention);
            test_logo_header_pendnant_style_with_labels("note".to_string(), DiagnosticStyle::NeedAttention);
        }

        fn test_logo_header_pendnant_style_with_labels(label: String, style: DiagnosticStyle) {
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
            assert!(style.style_eq(&styled_strings.get(0).unwrap().get(0).unwrap().style.as_ref().unwrap()));
            assert_eq!(styled_strings.get(0).unwrap().get(1).unwrap().text, ":");
        }
    }

    mod test_code_ctx_pendant {
        use crate::{
            pendant::CodeCtxPendant,
            tests::{check_styled_strings, get_code_position},
            Pendant,
        };
        use compiler_base_style::{ShaderFactory, diagnostic_style::DiagnosticStyle};
        use rustc_errors::styled_buffer::StyledBuffer;

        #[test]
        fn test_code_ctx_pendant() {
            let code_pos = get_code_position();
            let code_pendant = CodeCtxPendant::new(code_pos.clone());
            let shader = ShaderFactory::Diagnostic.get_shader();
            let mut sb = StyledBuffer::new();

            code_pendant.format(shader, &mut sb);
            let styled_strings = sb.render();
            let indent = code_pos.line.to_string().len() + 1;
            let col = code_pos.column.unwrap() as usize;

            let expected_texts = vec![
                vec![code_pos.info()],
                vec![format!("{:indent$}|", "")],
                vec![
                    format!("{:<indent$}", &code_pos.line),
                    "|    name = _name".to_string(),
                ],
                vec![format!("{:<indent$}|", ""), format!("{:>col$} ", format!("^{}",col))],
            ];
            let expected_styles = vec![
                vec![DiagnosticStyle::Url],
                vec![DiagnosticStyle::Normal],
                vec![DiagnosticStyle::Url, DiagnosticStyle::Normal],
                vec![DiagnosticStyle::Normal, DiagnosticStyle::NeedFix],
            ];

            check_styled_strings(
                &styled_strings,
                4,
                &vec![1, 1, 2, 2],
                &expected_texts,
                &expected_styles,
            );
        }
    }

    mod test_no_pendant {
        use compiler_base_style::{ShaderFactory, diagnostic_style::DiagnosticStyle};
        use rustc_errors::styled_buffer::StyledBuffer;

        use crate::{pendant::NoPendant, tests::check_styled_strings, Pendant};

        #[test]
        fn test_no_pendnant() {
            let no_pendant = NoPendant::new();
            let shader = ShaderFactory::Diagnostic.get_shader();
            let mut sb = StyledBuffer::new();

            no_pendant.format(shader, &mut sb);
            let styled_strings = sb.render();

            let expected_texts = vec![vec!["- ".to_string()]];
            let expected_styles = vec![vec![DiagnosticStyle::Normal]];

            check_styled_strings(
                &styled_strings,
                1,
                &vec![1],
                &expected_texts,
                &expected_styles,
            );
        }
    }
}

mod test_sentence {
    use compiler_base_style::{ShaderFactory, diagnostic_style::DiagnosticStyle};
    use rustc_errors::styled_buffer::StyledBuffer;

    use crate::{
        pendant::{CodeCtxPendant, HeaderPendant},
        Message, Sentence,
    };

    use super::{check_styled_strings, get_code_position};

    #[test]
    fn test_sentence_with_nopendant() {
        let sentence = Sentence::new_nopendant_sentence(Message::Str("test str".to_string()));
        let shader = ShaderFactory::Diagnostic.get_shader();
        let mut sb = StyledBuffer::new();
        sentence.format(shader, &mut sb);
        let styled_strings = sb.render();

        let expected_texts = vec![vec!["- test str".to_string()]];
        let expected_styles = vec![vec![DiagnosticStyle::Normal]];

        check_styled_strings(
            &styled_strings,
            1,
            &vec![1],
            &expected_texts,
            &expected_styles,
        );
    }

    #[test]
    fn test_sentence_with_headerpendant() {
        let diag_label = "test_label".to_string();
        let diag_code = Some("E1010".to_string());
        let header_pendant = HeaderPendant::new(diag_label.to_string(), diag_code);

        let sentence = Sentence::new_sentence_str(
            Box::new(header_pendant),
            Message::Str("test str".to_string()),
        );
        let shader = ShaderFactory::Diagnostic.get_shader();
        let mut sb = StyledBuffer::new();
        sentence.format(shader, &mut sb);
        let styled_strings = sb.render();

        let expected_texts = vec![vec![
            "test_label".to_string(),
            "[E1010]".to_string(),
            ":test str".to_string(),
        ]];
        let expected_styles = vec![vec![DiagnosticStyle::Normal, DiagnosticStyle::Helpful, DiagnosticStyle::Normal]];

        check_styled_strings(
            &styled_strings,
            1,
            &vec![3],
            &expected_texts,
            &expected_styles,
        );
    }

    #[test]
    fn test_sentence_with_code_ctx_sentence() {
        let code_pos = get_code_position();
        let code_pendant = CodeCtxPendant::new(code_pos.clone());
        let sentence = Sentence::new_sentence_str(
            Box::new(code_pendant),
            Message::Str("test str".to_string()),
        );

        let shader = ShaderFactory::Diagnostic.get_shader();
        let mut sb = StyledBuffer::new();

        sentence.format(shader, &mut sb);
        let styled_strings = sb.render();
        let indent = code_pos.line.to_string().len() + 1;
        let col = code_pos.column.unwrap() as usize;

        let expected_texts = vec![
            vec![code_pos.info()],
            vec![format!("{:indent$}|", "")],
            vec![
                format!("{:<indent$}", &code_pos.line),
                "|    name = _name".to_string(),
            ],
            vec![
                format!("{:<indent$}|", ""),
                format!("{:>col$}^ ", col),
                "test str".to_string(),
            ],
        ];
        let expected_styles = vec![
            vec![DiagnosticStyle::Url],
            vec![DiagnosticStyle::Normal],
            vec![DiagnosticStyle::Url, DiagnosticStyle::Normal],
            vec![DiagnosticStyle::Normal, DiagnosticStyle::NeedFix, DiagnosticStyle::Normal],
        ];

        check_styled_strings(
            &styled_strings,
            4,
            &vec![1, 1, 2, 3],
            &expected_texts,
            &expected_styles,
        );
    }
}

mod test_position {
    use crate::Position;

    #[test]
    fn test_dummy_pos() {
        let pos = Position::dummy_pos();
        assert_eq!(pos.filename, "".to_string());
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, None);
    }

    #[test]
    fn test_is_valid() {
        let mut pos = Position::dummy_pos();
        assert_eq!(pos.is_valid(), true);
        pos.line = 0;
        assert_eq!(pos.is_valid(), false);
    }

    #[test]
    fn test_less() {
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
    fn test_less_equal() {
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
    fn test_info() {
        let mut dummy_pos = Position::dummy_pos();
        assert_eq!(dummy_pos.info(), "---> File :1");
        let col = 12;
        dummy_pos.column = Some(col);
        let expected = format!("---> File :1:{}", col + 1);
        assert_eq!(dummy_pos.info(), expected);
    }
}

fn check_styled_strings(
    styled_strings: &Vec<Vec<StyledString>>,
    expected_line_count: usize,
    expected_string_counts: &Vec<usize>,
    expected_texts: &Vec<Vec<String>>,
    expected_styles: &Vec<Vec<DiagnosticStyle>>,
) {
    assert_eq!(styled_strings.len(), expected_line_count);
    assert_eq!(expected_texts.len(), expected_line_count);
    assert_eq!(expected_styles.len(), expected_line_count);

    for i in 0..expected_line_count {
        let styled_string_count = styled_strings.get(i).unwrap().len();
        assert_eq!(styled_string_count, *expected_string_counts.get(i).unwrap());
        assert_eq!(styled_string_count, expected_texts.get(i).unwrap().len());
        assert_eq!(styled_string_count, expected_styles.get(i).unwrap().len());

        for j in 0..styled_string_count {
            assert_eq!(
                styled_strings.get(i).unwrap().get(j).unwrap().text,
                expected_texts[i][j]
            );
            assert!(expected_styles[i][j].style_eq(&styled_strings.get(i).unwrap().get(j).unwrap().style.as_ref().unwrap()));
        }
    }
}

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


