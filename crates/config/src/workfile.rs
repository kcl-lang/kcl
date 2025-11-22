//! The config for IDE/LSP workspace config file `kcl.work'

use kcl_utils::path::PathPrefix;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use crate::modfile::KCL_WORK_FILE;
use anyhow::Result;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct WorkFile {
    pub workspaces: Vec<WorkSpace>,
    pub failed: HashMap<String, String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct WorkSpace {
    pub content: String,
    pub path: String,
    pub abs_path: String,
}

/// Load kcl work file from path
pub fn load_work_file<P: AsRef<Path> + std::fmt::Debug>(path: P) -> Result<WorkFile> {
    let file = if path.as_ref().is_dir() {
        let file_path = path.as_ref().join(KCL_WORK_FILE);
        std::fs::File::open(file_path)?
    } else if path.as_ref().is_file() {
        std::fs::File::open(&path)?
    } else {
        return Err(anyhow::anyhow!("kcl.work not found for {:?}", path));
    };

    let reader = BufReader::new(file);
    let mut workfile = WorkFile::default();
    for line in reader.lines() {
        if let Ok(line) = line {
            let mut directive = line.split_whitespace();
            if let Some(key) = directive.next() {
                match key {
                    "workspace" => {
                        if let Some(path) = directive.next() {
                            workfile.workspaces.push(WorkSpace {
                                content: line.clone(),
                                path: path.to_string(),
                                abs_path: "".to_string(),
                            });
                        }
                    }
                    _ => {
                        workfile.failed.insert(line, "Unknown keyword".to_string());
                    }
                }
            }
        }
    }
    Ok(workfile)
}

impl WorkFile {
    pub fn canonicalize(&mut self, root: PathBuf) {
        let mut new_workspaces = vec![];
        for workspace in self.workspaces.iter_mut() {
            let path = Path::new(&workspace.path);
            if !path.is_absolute() {
                let filepath = root.join(Path::new(&workspace.path));
                match filepath.canonicalize() {
                    Ok(path) => new_workspaces.push(WorkSpace {
                        content: workspace.content.clone(),
                        path: workspace.path.clone(),
                        abs_path: path.adjust_canonicalization(),
                    }),
                    Err(e) => {
                        self.failed.insert(
                            workspace.content.clone(),
                            format!("path canonicalize failed: {:?}", e),
                        );
                    }
                }
            } else {
                new_workspaces.push(WorkSpace {
                    content: workspace.content.clone(),
                    path: workspace.path.clone(),
                    abs_path: workspace.path.clone(),
                })
            };
        }
        self.workspaces = new_workspaces;
    }
}

#[cfg(test)]
mod workfile_test {
    use std::path::PathBuf;

    use crate::workfile::WorkSpace;

    use super::load_work_file;
    #[test]
    fn parse_workfile() {
        let path = "./src/testdata/";
        let workfile = load_work_file(path).unwrap();
        assert_eq!(
            workfile.workspaces,
            vec![
                WorkSpace {
                    content: "workspace ./a".to_string(),
                    path: "./a".to_string(),
                    abs_path: "".to_string()
                },
                WorkSpace {
                    content: "workspace ./b".to_string(),
                    path: "./b".to_string(),
                    abs_path: "".to_string()
                },
                WorkSpace {
                    content: "workspace ./c/d".to_string(),
                    path: "./c/d".to_string(),
                    abs_path: "".to_string()
                },
            ]
        );
    }

    #[test]
    fn parse_workfile1() {
        let path = "./src/testdata/kcl.work";
        let workfile = load_work_file(path).unwrap();
        assert_eq!(
            workfile.workspaces,
            vec![
                WorkSpace {
                    content: "workspace ./a".to_string(),
                    path: "./a".to_string(),
                    abs_path: "".to_string()
                },
                WorkSpace {
                    content: "workspace ./b".to_string(),
                    path: "./b".to_string(),
                    abs_path: "".to_string()
                },
                WorkSpace {
                    content: "workspace ./c/d".to_string(),
                    path: "./c/d".to_string(),
                    abs_path: "".to_string()
                },
            ]
        );
    }

    #[test]
    fn canonicalize_workfile() {
        let path = "./src/testdata/kcl.work";
        let mut workfile = load_work_file(path).unwrap();
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("testdata");
        let mut a = path.clone();
        a.push("a");

        let mut b = path.clone();
        b.push("b");

        let mut cd = path.clone();
        cd.push("c");
        cd.push("d");

        workfile.canonicalize(path);
        assert_eq!(
            workfile.workspaces,
            vec![
                WorkSpace {
                    content: "workspace ./a".to_string(),
                    path: "./a".to_string(),
                    abs_path: a.to_str().unwrap().to_string(),
                },
                WorkSpace {
                    content: "workspace ./b".to_string(),
                    path: "./b".to_string(),
                    abs_path: b.to_str().unwrap().to_string(),
                },
            ]
        );
        assert!(!workfile.failed.is_empty());
    }
}
