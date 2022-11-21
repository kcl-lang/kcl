use crate::langserver;
use crate::langserver::LineWord;
use kclvm_error::Position;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::{collections::HashMap, hash::Hash};

    fn check_line_to_words(code: &str, expect: Vec<LineWord>) {
        assert_eq!(langserver::line_to_words(code.to_string()), expect);
    }

    fn test_eq_list<T>(a: &[T], b: &[T]) -> bool
    where
        T: Eq + Hash,
    {
        fn count<T>(items: &[T]) -> HashMap<&T, usize>
        where
            T: Eq + Hash,
        {
            let mut cnt = HashMap::new();
            for i in items {
                *cnt.entry(i).or_insert(0) += 1
            }
            cnt
        }

        count(a) == count(b)
    }

    #[test]
    fn test_line_to_words() {
        let datas = vec![
            "alice_first_name = \"alice\"",
            "0lice_first_name = \"alic0\"",
            "alice = p.Parent { name: \"alice\" }",
        ];
        let expect = vec![
            vec![
                LineWord {
                    startpos: 0,
                    endpos: 16,
                    word: "alice_first_name".to_string(),
                },
                LineWord {
                    startpos: 20,
                    endpos: 25,
                    word: "alice".to_string(),
                },
            ],
            vec![LineWord {
                startpos: 20,
                endpos: 25,
                word: "alic0".to_string(),
            }],
            vec![
                LineWord {
                    startpos: 0,
                    endpos: 5,
                    word: "alice".to_string(),
                },
                LineWord {
                    startpos: 8,
                    endpos: 9,
                    word: "p".to_string(),
                },
                LineWord {
                    startpos: 10,
                    endpos: 16,
                    word: "Parent".to_string(),
                },
                LineWord {
                    startpos: 19,
                    endpos: 23,
                    word: "name".to_string(),
                },
                LineWord {
                    startpos: 26,
                    endpos: 31,
                    word: "alice".to_string(),
                },
            ],
        ];
        for i in 0..datas.len() {
            check_line_to_words(datas[i], expect[i].clone());
        }
    }

    #[test]
    fn test_word_at_pos() {
        // use std::env;
        // let parent_path = env::current_dir().unwrap();
        // println!("The current directory is {}", parent_path.display());
        let path_prefix = "./src/langserver/".to_string();
        let datas = vec![
            Position {
                filename: (path_prefix.clone() + "test_data/inherit.k"),
                line: 0,
                column: Some(0),
            },
            Position {
                filename: (path_prefix.clone() + "test_data/inherit.k"),
                line: 1,
                column: Some(5),
            },
            Position {
                filename: (path_prefix.clone() + "test_data/inherit.k"),
                line: 3,
                column: Some(7),
            },
            Position {
                filename: (path_prefix.clone() + "test_data/inherit.k"),
                line: 3,
                column: Some(10),
            },
            Position {
                filename: (path_prefix.clone() + "test_data/inherit.k"),
                line: 4,
                column: Some(8),
            },
            Position {
                filename: (path_prefix + "test_data/inherit.k"),
                line: 4,
                column: Some(100),
            },
        ];
        let expect = vec![
            Some("schema".to_string()),
            Some("name".to_string()),
            Some("Son".to_string()),
            None,
            None,
            None,
        ];
        for i in 0..datas.len() {
            assert_eq!(langserver::word_at_pos(datas[i].clone()), expect[i]);
        }
    }

    #[test]
    fn test_match_word() {
        let path = "./src/langserver/test_data/test_word_workspace".to_string();
        let datas = vec![String::from("Son")];
        let except = vec![vec![
            Position {
                filename: String::from(
                    "./src/langserver/test_data/test_word_workspace/inherit_pkg.k",
                ),
                line: 2,
                column: Some(7),
            },
            Position {
                filename: String::from("./src/langserver/test_data/test_word_workspace/inherit.k"),
                line: 3,
                column: Some(7),
            },
            Position {
                filename: String::from("./src/langserver/test_data/test_word_workspace/inherit.k"),
                line: 7,
                column: Some(16),
            },
        ]];
        for i in 0..datas.len() {
            assert!(test_eq_list(
                &langserver::match_word(path.clone(), datas[i].clone()),
                &except[i]
            ));
        }
    }

    #[test]
    fn test_word_map() {
        let path = "./src/langserver/test_data/test_word_workspace_map".to_string();
        let mut mp = langserver::word_map::WorkSpaceWordMap::new(path);
        mp.build();
        let _res = fs::rename(
            "./src/langserver/test_data/test_word_workspace_map/inherit_pkg.k",
            "./src/langserver/test_data/test_word_workspace_map/inherit_bak.k",
        );
        mp.rename_file(
            "./src/langserver/test_data/test_word_workspace_map/inherit_pkg.k".to_string(),
            "./src/langserver/test_data/test_word_workspace_map/inherit_bak.k".to_string(),
        );
        mp.delete_file("./src/langserver/test_data/test_word_workspace_map/inherit.k".to_string());
        let _res = fs::rename(
            "./src/langserver/test_data/test_word_workspace_map/inherit_bak.k",
            "./src/langserver/test_data/test_word_workspace_map/inherit_pkg.k",
        );

        let except = vec![Position {
            filename: String::from(
                "./src/langserver/test_data/test_word_workspace_map/inherit_bak.k",
            ),
            line: 2,
            column: Some(7),
        }];
        assert_eq!(mp.get(&String::from("Son")), Some(except));
    }
}
