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
