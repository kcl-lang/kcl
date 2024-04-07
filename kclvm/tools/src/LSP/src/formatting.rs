use kclvm_tools::format::{format_source, FormatOptions};
use lsp_types::{Position, Range, TextEdit};

pub(crate) fn format(
    file: String,
    src: String,
    range: Option<Range>,
) -> anyhow::Result<Option<Vec<TextEdit>>> {
    let (source, is_formatted) = format_source(
        &file,
        &src,
        &FormatOptions {
            omit_errors: true,
            ..Default::default()
        },
    )
    .map_err(|err| anyhow::anyhow!("Formatting failed: {}", err))?;
    if is_formatted {
        Ok(Some(vec![TextEdit {
            range: range.unwrap_or(Range::new(
                Position::new(0, 0),
                Position::new(u32::MAX, u32::MAX),
            )),
            new_text: source,
        }]))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use std::{ops::Index, path::PathBuf};

    use super::format;
    use lsp_types::{Position, Range, TextEdit};
    use proc_macro_crate::bench_test;

    use crate::{from_lsp::text_range, tests::compile_test_file};

    #[test]
    fn format_signle_file_test() {
        const FILE_INPUT_SUFFIX: &str = ".input";
        const FILE_OUTPUT_SUFFIX: &str = ".golden";
        const TEST_CASES: &[&str; 17] = &[
            "assert",
            "check",
            "blankline",
            "breakline",
            "codelayout",
            "collection_if",
            "comment",
            "comp_for",
            // "empty",
            "import",
            "indent",
            "inline_comment",
            "lambda",
            "quant",
            "schema",
            "string",
            "type_alias",
            "unary",
        ];

        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let test_file = path;
        let test_dir = test_file
            .parent()
            .unwrap()
            .join("format")
            .join("test_data")
            .join("format_data");
        for case in TEST_CASES {
            let test_file = test_dir
                .join(format!("{}{}", case, FILE_INPUT_SUFFIX))
                .to_str()
                .unwrap()
                .to_string();
            let test_src = std::fs::read_to_string(&test_file).unwrap();
            let got = format(test_file.to_string(), test_src, None)
                .unwrap()
                .unwrap();
            let data_output = std::fs::read_to_string(
                test_dir
                    .join(format!("{}{}", case, FILE_OUTPUT_SUFFIX))
                    .to_str()
                    .unwrap(),
            )
            .unwrap();

            #[cfg(target_os = "windows")]
            let data_output = data_output.replace("\r\n", "\n");

            let expect = vec![TextEdit {
                range: Range::new(Position::new(0, 0), Position::new(u32::MAX, u32::MAX)),
                new_text: data_output,
            }];
            println!("{:?}", test_file);
            assert_eq!(expect, got);
        }

        // empty test case, without change after fmt
        let test_file = test_dir
            .join(format!("{}{}", "empty", FILE_INPUT_SUFFIX))
            .to_str()
            .unwrap()
            .to_string();
        let test_src = std::fs::read_to_string(&test_file).unwrap();
        let got = format(test_file, test_src, None).unwrap();
        assert_eq!(got, None)
    }

    #[test]
    #[bench_test]
    fn format_range_test() {
        let (file, _program, _, _gs) = compile_test_file("src/test_data/format/format_range.k");
        let lsp_range = Range::new(Position::new(0, 0), Position::new(11, 0));
        let text = std::fs::read_to_string(file.clone()).unwrap();

        let range = text_range(&text, lsp_range);
        let src = text.index(range);

        let got = format(file, src.to_owned(), Some(lsp_range))
            .unwrap()
            .unwrap();

        let expected = vec![TextEdit {
            range: lsp_range,
            new_text: "a = 1\nb = 2\nc = 3\n".to_string(),
        }];
        assert_eq!(got, expected)
    }
}
