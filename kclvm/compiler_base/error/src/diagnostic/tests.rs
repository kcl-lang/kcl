use super::style::DiagnosticStyle;
use rustc_errors::styled_buffer::StyledString;

mod test_pendant {
    mod test_label_pendant {
        use crate::diagnostic::{
            pendant::LabelPendant, style::DiagnosticStyle, tests::check_styled_strings, Formatter,
        };
        use rustc_errors::styled_buffer::StyledBuffer;

        #[test]
        fn test_label_pendnant() {
            let diag_label = "test_label".to_string();
            let diag_code = Some("E1010".to_string());

            let mut label_pendant = LabelPendant::new(diag_label.to_string(), diag_code);
            label_pendant.set_logo("LOGO".to_string());

            let mut sb = StyledBuffer::new();
            label_pendant.format(&mut sb);

            let styled_strings = sb.render();
            let expected_texts = vec![vec![
                "LOGO".to_string(),
                " test_label".to_string(),
                "[E1010]".to_string(),
                ":".to_string(),
            ]];
            let expected_styles = vec![vec![
                DiagnosticStyle::Logo,
                DiagnosticStyle::NoStyle,
                DiagnosticStyle::Helpful,
                DiagnosticStyle::NoStyle,
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
        fn test_label_pendnant_style() {
            test_logo_label_pendnant_style_with_labels(
                "error".to_string(),
                DiagnosticStyle::NeedFix,
            );
            test_logo_label_pendnant_style_with_labels(
                "warning".to_string(),
                DiagnosticStyle::NeedAttention,
            );
            test_logo_label_pendnant_style_with_labels(
                "help".to_string(),
                DiagnosticStyle::Helpful,
            );
            test_logo_label_pendnant_style_with_labels(
                "note".to_string(),
                DiagnosticStyle::Important,
            );
        }

        fn test_logo_label_pendnant_style_with_labels(label: String, style: DiagnosticStyle) {
            let mut sb = StyledBuffer::new();
            let label_pendant = LabelPendant::new(label.to_string(), None);
            label_pendant.format(&mut sb);
            let styled_strings = sb.render();
            assert_eq!(styled_strings.len(), 1);
            assert_eq!(styled_strings.get(0).unwrap().len(), 2);

            assert_eq!(
                styled_strings.get(0).unwrap().get(0).unwrap().text,
                label.to_string()
            );
            assert!(
                style
                    == *styled_strings
                        .get(0)
                        .unwrap()
                        .get(0)
                        .unwrap()
                        .style
                        .as_ref()
                        .unwrap()
            );
            assert_eq!(styled_strings.get(0).unwrap().get(1).unwrap().text, ":");
        }
    }
}

mod test_sentence {
    use rustc_errors::styled_buffer::StyledBuffer;

    use crate::diagnostic::{pendant::LabelPendant, style::DiagnosticStyle, Formatter, Sentence};

    use super::check_styled_strings;

    #[test]
    fn test_sentence_with_labelpendant() {
        let diag_label = "test_label".to_string();
        let diag_code = Some("E1010".to_string());
        let label_pendant = LabelPendant::new(diag_label.to_string(), diag_code);

        let sentence =
            Sentence::new_sentence_str(Box::new(label_pendant), Box::new("test str".to_string()));
        let mut sb = StyledBuffer::new();
        sentence.format(&mut sb);
        let styled_strings = sb.render();

        let expected_texts = vec![vec![
            "test_label".to_string(),
            "[E1010]".to_string(),
            ":test str".to_string(),
        ]];
        let expected_styles = vec![vec![
            DiagnosticStyle::NoStyle,
            DiagnosticStyle::Helpful,
            DiagnosticStyle::NoStyle,
        ]];

        check_styled_strings(
            &styled_strings,
            1,
            &vec![3],
            &expected_texts,
            &expected_styles,
        );
    }
}

fn check_styled_strings(
    styled_strings: &Vec<Vec<StyledString<DiagnosticStyle>>>,
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
            assert!(
                expected_styles[i][j]
                    == *styled_strings
                        .get(i)
                        .unwrap()
                        .get(j)
                        .unwrap()
                        .style
                        .as_ref()
                        .unwrap()
            );
        }
    }
}
