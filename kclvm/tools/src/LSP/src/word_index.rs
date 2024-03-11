use kclvm_driver::get_kcl_files;
use kclvm_span::symbol::reserved;
use lsp_types::{Location, Position, Range, Url};

use std::{collections::HashMap, path::Path};

/// WordIndex represents the correspondence between Word and all its positions
pub type WordIndexMap = HashMap<String, Vec<Location>>;
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

/// Scan and build a (word -> Locations) index map.
pub(crate) fn build_word_index<P: AsRef<Path>>(
    path: P,
    prune: bool,
) -> anyhow::Result<WordIndexMap> {
    let files = get_kcl_files(path, true)?;
    build_word_index_with_paths(&files, prune)
}

pub(crate) fn build_word_index_with_paths(
    paths: &[String],
    prune: bool,
) -> anyhow::Result<WordIndexMap> {
    let mut index: WordIndexMap = HashMap::new();
    for p in paths {
        // str path to url
        if let Ok(url) = Url::from_file_path(p) {
            // read file content and save the word to word index
            let text = std::fs::read_to_string(p)?;
            for (key, values) in build_word_index_with_content(&text, &url, prune) {
                index.entry(key).or_default().extend(values);
            }
        }
    }
    Ok(index)
}

pub(crate) fn build_word_index_with_content(content: &str, url: &Url, prune: bool) -> WordIndexMap {
    let mut index: WordIndexMap = HashMap::new();
    let mut in_docstring = false;
    for (li, line) in content.lines().enumerate() {
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
                .extend(values.iter().map(|w| Location {
                    uri: url.clone(),
                    range: Range {
                        start: Position::new(li as u32, w.start_col),
                        end: Position::new(li as u32, w.end_col),
                    },
                }));
        }
    }
    index
}

pub(crate) fn word_index_add(from: &mut WordIndexMap, add: WordIndexMap) {
    for (key, value) in add {
        from.entry(key).or_default().extend(value);
    }
}

pub(crate) fn word_index_subtract(from: &mut WordIndexMap, remove: WordIndexMap) {
    for (key, value) in remove {
        for v in value {
            from.entry(key.clone()).and_modify(|locations| {
                locations.retain(|loc| loc != &v);
            });
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
    use super::{
        build_word_index, build_word_index_with_content, line_to_words, word_index_add,
        word_index_subtract, Word,
    };
    use lsp_types::{Location, Position, Range, Url};
    use std::collections::HashMap;
    use std::path::PathBuf;
    #[test]
    fn test_build_word_index() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut path = root.clone();
        path.push("src/test_data/find_refs_test/main.k");

        let url = lsp_types::Url::from_file_path(path.clone()).unwrap();
        let path = path.to_str().unwrap();
        let expect: HashMap<String, Vec<Location>> = vec![
            (
                "a".to_string(),
                vec![
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 1),
                        },
                    },
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(1, 4),
                            end: Position::new(1, 5),
                        },
                    },
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(2, 4),
                            end: Position::new(2, 5),
                        },
                    },
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(12, 14),
                            end: Position::new(12, 15),
                        },
                    },
                ],
            ),
            (
                "c".to_string(),
                vec![Location {
                    uri: url.clone(),
                    range: Range {
                        start: Position::new(2, 0),
                        end: Position::new(2, 1),
                    },
                }],
            ),
            (
                "b".to_string(),
                vec![Location {
                    uri: url.clone(),
                    range: Range {
                        start: Position::new(1, 0),
                        end: Position::new(1, 1),
                    },
                }],
            ),
            (
                "n".to_string(),
                vec![
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(8, 4),
                            end: Position::new(8, 5),
                        },
                    },
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(11, 4),
                            end: Position::new(11, 5),
                        },
                    },
                ],
            ),
            (
                "b".to_string(),
                vec![Location {
                    uri: url.clone(),
                    range: Range {
                        start: Position::new(1, 0),
                        end: Position::new(1, 1),
                    },
                }],
            ),
            (
                "Name".to_string(),
                vec![
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(4, 7),
                            end: Position::new(4, 11),
                        },
                    },
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(8, 7),
                            end: Position::new(8, 11),
                        },
                    },
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(11, 7),
                            end: Position::new(11, 11),
                        },
                    },
                ],
            ),
            (
                "name".to_string(),
                vec![
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(5, 4),
                            end: Position::new(5, 8),
                        },
                    },
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(12, 8),
                            end: Position::new(12, 12),
                        },
                    },
                ],
            ),
            (
                "demo".to_string(),
                vec![Location {
                    uri: url.clone(),
                    range: Range {
                        start: Position::new(0, 5),
                        end: Position::new(0, 9),
                    },
                }],
            ),
            (
                "Person".to_string(),
                vec![
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(7, 7),
                            end: Position::new(7, 13),
                        },
                    },
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(10, 5),
                            end: Position::new(10, 11),
                        },
                    },
                ],
            ),
            (
                "p2".to_string(),
                vec![Location {
                    uri: url.clone(),
                    range: Range {
                        start: Position::new(10, 0),
                        end: Position::new(10, 2),
                    },
                }],
            ),
        ]
        .into_iter()
        .collect();
        match build_word_index(path, true) {
            Ok(actual) => {
                assert_eq!(expect, actual)
            }
            Err(_) => unreachable!("build word index failed. expect: {:?}", expect),
        }
    }

    #[test]
    fn test_word_index_add() {
        let loc1 = Location {
            uri: Url::parse("file:///path/to/file.k").unwrap(),
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(0, 4),
            },
        };
        let loc2 = Location {
            uri: Url::parse("file:///path/to/file.k").unwrap(),
            range: Range {
                start: Position::new(1, 0),
                end: Position::new(1, 4),
            },
        };
        let mut from = HashMap::from([("name".to_string(), vec![loc1.clone()])]);
        let add = HashMap::from([("name".to_string(), vec![loc2.clone()])]);
        word_index_add(&mut from, add);
        assert_eq!(
            from,
            HashMap::from([("name".to_string(), vec![loc1.clone(), loc2.clone()],)])
        );
    }

    #[test]
    fn test_word_index_subtract() {
        let loc1 = Location {
            uri: Url::parse("file:///path/to/file.k").unwrap(),
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(0, 4),
            },
        };
        let loc2 = Location {
            uri: Url::parse("file:///path/to/file.k").unwrap(),
            range: Range {
                start: Position::new(1, 0),
                end: Position::new(1, 4),
            },
        };
        let mut from = HashMap::from([("name".to_string(), vec![loc1.clone(), loc2.clone()])]);
        let remove = HashMap::from([("name".to_string(), vec![loc2.clone()])]);
        word_index_subtract(&mut from, remove);
        assert_eq!(
            from,
            HashMap::from([("name".to_string(), vec![loc1.clone()],)])
        );
    }

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

    #[test]
    fn test_build_word_index_for_file_content() {
        let content = r#"schema Person:
    """
    This is a docstring.
    Person is a schema which defines a person's name and age.
    """
    name: str # name must not be empty
    # age is a positive integer
    age: int
"#;
        let mock_url = Url::parse("file:///path/to/file.k").unwrap();
        let expects: HashMap<String, Vec<Location>> = vec![
            (
                "Person".to_string(),
                vec![Location {
                    uri: mock_url.clone(),
                    range: Range {
                        start: Position::new(0, 7),
                        end: Position::new(0, 13),
                    },
                }],
            ),
            (
                "name".to_string(),
                vec![Location {
                    uri: mock_url.clone(),
                    range: Range {
                        start: Position::new(5, 4),
                        end: Position::new(5, 8),
                    },
                }],
            ),
            (
                "age".to_string(),
                vec![Location {
                    uri: mock_url.clone(),
                    range: Range {
                        start: Position::new(7, 4),
                        end: Position::new(7, 7),
                    },
                }],
            ),
        ]
        .into_iter()
        .collect();

        let got = build_word_index_with_content(content, &mock_url.clone(), true);
        assert_eq!(expects, got)
    }
}
