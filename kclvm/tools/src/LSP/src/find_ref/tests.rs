use crate::find_ref;
use crate::find_ref::LineWord;
use kclvm_error::Position;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::{collections::HashMap, hash::Hash};

    fn check_line_to_words(code: &str, expect: Vec<LineWord>) {
        assert_eq!(find_ref::line_to_words(code.to_string()), expect);
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
        let path_prefix = "./src/find_ref/".to_string();
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
            assert_eq!(find_ref::word_at_pos(datas[i].clone()), expect[i]);
        }
    }

    fn test_word_workspace() -> String {
        Path::new(".")
            .join("src")
            .join("find_ref")
            .join("test_data")
            .join("test_word_workspace")
            .display()
            .to_string()
    }

    #[test]
    fn test_match_word() {
        let path = test_word_workspace();
        let datas = vec![String::from("Son")];
        let except = vec![vec![
            Position {
                filename: Path::new(&test_word_workspace())
                    .join("inherit_pkg.k")
                    .display()
                    .to_string(),
                line: 2,
                column: Some(7),
            },
            Position {
                filename: Path::new(&test_word_workspace())
                    .join("inherit.k")
                    .display()
                    .to_string(),
                line: 3,
                column: Some(7),
            },
            Position {
                filename: Path::new(&test_word_workspace())
                    .join("inherit.k")
                    .display()
                    .to_string(),
                line: 7,
                column: Some(16),
            },
        ]];
        for i in 0..datas.len() {
            assert!(test_eq_list(
                &find_ref::match_word(path.clone(), datas[i].clone()),
                &except[i]
            ));
        }
    }

    fn test_word_workspace_map() -> String {
        Path::new(".")
            .join("src")
            .join("find_ref")
            .join("test_data")
            .join("test_word_workspace_map")
            .display()
            .to_string()
    }

    #[test]
    fn test_word_map() {
        let path = test_word_workspace_map();
        let mut mp = find_ref::word_map::WorkSpaceWordMap::new(path);
        mp.build();
        let _res = fs::rename(
            Path::new(&test_word_workspace_map())
                .join("inherit_pkg.k")
                .display()
                .to_string(),
            Path::new(&test_word_workspace_map())
                .join("inherit_bak.k")
                .display()
                .to_string(),
        );
        mp.rename_file(
            Path::new(&test_word_workspace_map())
                .join("inherit_pkg.k")
                .display()
                .to_string(),
            Path::new(&test_word_workspace_map())
                .join("inherit_bak.k")
                .display()
                .to_string(),
        );
        mp.delete_file(
            Path::new(&test_word_workspace_map())
                .join("inherit.k")
                .display()
                .to_string(),
        );
        let _res = fs::rename(
            Path::new(&test_word_workspace_map())
                .join("inherit_bak.k")
                .display()
                .to_string(),
            Path::new(&test_word_workspace_map())
                .join("inherit_pkg.k")
                .display()
                .to_string(),
        );

        let except = vec![Position {
            filename: Path::new(&test_word_workspace_map())
                .join("inherit_bak.k")
                .display()
                .to_string(),
            line: 2,
            column: Some(7),
        }];
        assert_eq!(mp.get(&String::from("Son")), Some(except));
    }
}
