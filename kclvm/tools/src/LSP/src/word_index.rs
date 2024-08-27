use kclvm_span::symbol::reserved;
use lsp_types::{Position, Range};
use std::collections::HashMap;

/// WordIndex represents the correspondence between Word and all its positions
/// VirtualWordIndexMap represents the correspondence between Word and all its positions
pub type VirtualWordIndexMap = HashMap<String, Vec<VirtualLocation>>;
/// WordMap represents the correspondence between text and all its positions
pub type WordMap = HashMap<String, Vec<Word>>;

/// Word describes an arbitrary word in a certain line including
/// start position, end position and the word itself.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Word {
    start_col: u32,
    end_col: u32,
    word: String,
}

impl Word {
    fn new(start_col: u32, end_col: u32, word: String) -> Self {
        Self {
            start_col,
            end_col,
            word,
        }
    }
}

/// Split one line into identifier words.
fn line_to_words(text: String, prune: bool) -> WordMap {
    let mut result = HashMap::new();
    let mut chars: Vec<char> = text.chars().collect();
    chars.push('\n');
    let mut start_pos = usize::MAX;
    let mut continue_pos = usize::MAX - 1; // avoid overflow
    let mut prev_word = false;
    let mut words: Vec<Word> = vec![];
    for (i, ch) in chars.iter().enumerate() {
        if prune && *ch == '#' {
            break;
        }
        let is_id_start = rustc_lexer::is_id_start(*ch);
        let is_id_continue = rustc_lexer::is_id_continue(*ch);
        // If the character is valid identifier start and the previous character is not valid identifier continue, mark the start position.
        if is_id_start && !prev_word {
            start_pos = i;
        }
        if is_id_continue {
            // Continue searching for the end position.
            if start_pos != usize::MAX {
                continue_pos = i;
            }
        } else {
            // Find out the end position.
            if continue_pos + 1 == i {
                let word = chars[start_pos..i].iter().collect::<String>().clone();
                // Skip word if it should be pruned
                if !prune || !reserved::is_reserved_word(&word) {
                    words.push(Word::new(start_pos as u32, i as u32, word));
                }
            }
            // Reset the start position.
            start_pos = usize::MAX;
        }
        prev_word = is_id_continue;
    }

    for w in words {
        result.entry(w.word.clone()).or_insert(Vec::new()).push(w);
    }
    result
}

/// VirtualLocation represents a location inside a resource, such as a line inside a text file.
#[allow(unused)]
pub(crate) struct VirtualLocation {
    pub(crate) filepath: String,
    pub(crate) range: Range,
}

#[allow(unused)]
pub(crate) fn build_virtual_word_index(
    source_codes: HashMap<String, String>,
    prune: bool,
) -> anyhow::Result<VirtualWordIndexMap> {
    let mut index: VirtualWordIndexMap = HashMap::new();
    for (filepath, content) in source_codes.iter() {
        for (key, values) in build_virtual_word_index_with_file_content(
            filepath.to_string(),
            content.to_string(),
            prune,
        ) {
            index.entry(key).or_default().extend(values);
        }
    }
    Ok(index)
}

#[allow(unused)]
fn build_virtual_word_index_with_file_content(
    filepath: String,
    content: String,
    prune: bool,
) -> VirtualWordIndexMap {
    let mut index: HashMap<String, Vec<VirtualLocation>> = HashMap::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut in_docstring = false;
    for (li, line) in lines.into_iter().enumerate() {
        if prune && !in_docstring && line.trim_start().starts_with("\"\"\"") {
            in_docstring = true;
            continue;
        }
        if prune && in_docstring {
            if line.trim_end().ends_with("\"\"\"") {
                in_docstring = false;
            }
            continue;
        }
        let words = line_to_words(line.to_string(), prune);
        for (key, values) in words {
            index
                .entry(key)
                .or_default()
                .extend(values.iter().map(|w| VirtualLocation {
                    filepath: filepath.clone(),
                    range: Range {
                        start: Position::new(li as u32, w.start_col),
                        end: Position::new(li as u32, w.end_col),
                    },
                }));
        }
    }
    index
}

#[cfg(test)]
mod tests {
    use super::{line_to_words, Word};
    use std::collections::HashMap;

    #[test]
    fn test_line_to_words() {
        let lines = [
            "schema Person:",
            "name. name again",
            "some_word word !word",
            "# this line is a single-line comment",
            "name # end of line comment",
        ];

        let expects: Vec<HashMap<String, Vec<Word>>> = vec![
            vec![(
                "Person".to_string(),
                vec![Word {
                    start_col: 7,
                    end_col: 13,
                    word: "Person".to_string(),
                }],
            )]
            .into_iter()
            .collect(),
            vec![
                (
                    "name".to_string(),
                    vec![
                        Word {
                            start_col: 0,
                            end_col: 4,
                            word: "name".to_string(),
                        },
                        Word {
                            start_col: 6,
                            end_col: 10,
                            word: "name".to_string(),
                        },
                    ],
                ),
                (
                    "again".to_string(),
                    vec![Word {
                        start_col: 11,
                        end_col: 16,
                        word: "again".to_string(),
                    }],
                ),
            ]
            .into_iter()
            .collect(),
            vec![
                (
                    "some_word".to_string(),
                    vec![Word {
                        start_col: 0,
                        end_col: 9,
                        word: "some_word".to_string(),
                    }],
                ),
                (
                    "word".to_string(),
                    vec![
                        Word {
                            start_col: 10,
                            end_col: 14,
                            word: "word".to_string(),
                        },
                        Word {
                            start_col: 16,
                            end_col: 20,
                            word: "word".to_string(),
                        },
                    ],
                ),
            ]
            .into_iter()
            .collect(),
            HashMap::new(),
            vec![(
                "name".to_string(),
                vec![Word {
                    start_col: 0,
                    end_col: 4,
                    word: "name".to_string(),
                }],
            )]
            .into_iter()
            .collect(),
        ];
        for i in 0..lines.len() {
            let got = line_to_words(lines[i].to_string(), true);
            assert_eq!(expects[i], got)
        }
    }
}
