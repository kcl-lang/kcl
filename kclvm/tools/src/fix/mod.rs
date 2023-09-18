mod replace;
use anyhow::Error;
use kclvm_error::{diagnostic::Range as KCLRange, Diagnostic};
use std::collections::HashMap;
use std::fs;
use std::ops::Range;

pub struct CodeFix {
    data: replace::Data,
}

impl CodeFix {
    pub fn new(s: &str) -> CodeFix {
        CodeFix {
            data: replace::Data::new(s.as_bytes()),
        }
    }

    pub fn apply(&mut self, suggestion: &Suggestion) -> Result<(), Error> {
        let snippet = &suggestion.replacement.snippet;
        self.data.replace_range(
            snippet.range.start,
            snippet.range.end.saturating_sub(1),
            suggestion.replacement.replacement.as_bytes(),
        )?;
        Ok(())
    }

    pub fn finish(&self) -> Result<String, Error> {
        Ok(String::from_utf8(self.data.to_vec())?)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// An error/warning and possible solutions for fixing it
pub struct Suggestion {
    pub message: String,
    pub replacement: Replacement,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Replacement {
    pub snippet: Snippet,
    pub replacement: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Snippet {
    pub file_name: String,
    pub range: Range<usize>,
}

pub fn diag_to_suggestion(
    diag: Diagnostic,
    files: &mut HashMap<String, String>,
) -> Vec<Suggestion> {
    let mut suggestions = vec![];

    for msg in &diag.messages {
        if let Some(replace) = &msg.suggested_replacement {
            let file_name = msg.range.0.filename.clone();
            let src = match files.get(&file_name) {
                Some(src) => src.clone(),
                None => {
                    let src = fs::read_to_string(&file_name).unwrap();
                    files.insert(file_name, src.clone());
                    src
                }
            };
            suggestions.push(Suggestion {
                message: msg.message.clone(),
                replacement: Replacement {
                    snippet: Snippet {
                        file_name: msg.range.0.filename.clone(),
                        range: text_range(src.as_str(), &msg.range),
                    },
                    replacement: replace.clone(),
                },
            });
        }
    }
    suggestions
}

/// Converts the given lsp range to `Range`
pub(crate) fn text_range(text: &str, range: &KCLRange) -> Range<usize> {
    let mut lines_length = vec![];
    let lines_text: Vec<&str> = text.split('\n').collect();
    let mut pre_total_length = 0;

    for line in &lines_text {
        lines_length.push(pre_total_length);
        pre_total_length += line.len() + "\n".len();
    }

    let start =
        lines_length.get(range.0.line as usize - 1).unwrap() + range.0.column.unwrap_or(0) as usize;
    let mut end =
        lines_length.get(range.1.line as usize - 1).unwrap() + range.1.column.unwrap_or(0) as usize;
    if let Some(ch) = text.chars().nth(end) {
        if ch == '\n' {
            end += 1;
        }
    }
    Range { start, end }
}

pub fn fix(diags: Vec<Diagnostic>) -> Result<(), Error> {
    let mut suggestions = vec![];
    let mut source_code = HashMap::new();
    for diag in diags {
        suggestions.extend(diag_to_suggestion(diag, &mut source_code))
    }

    let mut files = HashMap::new();
    for suggestion in suggestions {
        let file = suggestion.replacement.snippet.file_name.clone();
        files.entry(file).or_insert_with(Vec::new).push(suggestion);
    }

    for (source_file, suggestions) in &files {
        println!("fix file: {:?}", source_file);
        let source = fs::read_to_string(source_file)?;
        let mut fix = CodeFix::new(&source);
        for suggestion in suggestions.iter() {
            if let Err(e) = fix.apply(suggestion) {
                eprintln!("Failed to apply suggestion to {}: {}", source_file, e);
            }
        }
        let fixes = fix.finish()?;
        fs::write(source_file, fixes)?;
    }
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use crate::lint::lint_files;

//     use super::*;

//     #[test]
//     fn test_aa() -> Result<(), Error> {
//         let mut diags: Vec<Diagnostic> = vec![];
//         let (errors, warnings) = lint_files(&["/Users/zz/code/KCLVM/aa.k"], None);
//         diags.extend(warnings);
//         let mut suggestions = vec![];
//         let mut source_code = HashMap::new();
//         for diag in diags {
//             suggestions.extend(diag_to_suggestion(diag, &mut source_code))
//         }

//         let mut files = HashMap::new();
//         for suggestion in suggestions {
//             let file = suggestion.replacement.snippet.file_name.clone();
//             files.entry(file).or_insert_with(Vec::new).push(suggestion);
//         }

//         for (source_file, suggestions) in &files {
//             let source = fs::read_to_string(source_file)?;
//             let mut fix = CodeFix::new(&source);
//             for suggestion in suggestions.iter() {
//                 if let Err(e) = fix.apply(suggestion) {
//                     eprintln!("Failed to apply suggestion to {}: {}", source_file, e);
//                 }
//             }
//             let fixes = fix.finish()?;
//             fs::write(source_file, fixes)?;
//         }

//         Ok(())
//     }
// }
