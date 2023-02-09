use std::path::PathBuf;

use anyhow::{Context, Result};

const CARGO_DIR: &str = env!("CARGO_MANIFEST_DIR");
const REL_PATH: &str = "src/util/test_datas";
const FILE_TEST_CASES: &[&str] = &["test"];

const FILE_EXTENSIONS: &[&str] = &[".json", ".yaml"];

const JSON_STR_TEST_CASES: &[&str] = &[r#"{
    "name": "John Doe",
    "age": 43,
    "address": {
        "street": "10 Downing Street",
        "city": "London"
    },
    "phones": [
        "+44 1234567",
        "+44 2345678"
    ]
}
"#];

const YAML_STR_TEST_CASES: &[&str] = &[r#"languages:
  - Ruby
  - Perl
  - Python 
websites:
  YAML: yaml.org 
  Ruby: ruby-lang.org 
  Python: python.org 
  Perl: use.perl.org
"#];

fn construct_full_path(path: &str) -> Result<String> {
    let mut cargo_file_path = PathBuf::from(CARGO_DIR);
    cargo_file_path.push(REL_PATH);
    cargo_file_path.push(path);
    Ok(cargo_file_path
        .to_str()
        .with_context(|| format!("No such file or directory '{}'", path))?
        .to_string())
}

mod test_loader {
    mod test_data_loader {
        use regex::Regex;

        use crate::util::{
            loader::{DataLoader, Loader, LoaderKind},
            tests::{
                construct_full_path, FILE_EXTENSIONS, FILE_TEST_CASES, JSON_STR_TEST_CASES,
                YAML_STR_TEST_CASES,
            },
        };

        fn data_loader_from_file(loader_kind: LoaderKind, file_path: &str) -> DataLoader {
            let test_case_path = construct_full_path(file_path).unwrap();

            DataLoader::new_with_file_path(loader_kind, &test_case_path).unwrap()
        }

        fn data_loader_from_str(loader_kind: LoaderKind, s: &str) -> DataLoader {
            DataLoader::new_with_str(loader_kind, s).unwrap()
        }

        #[test]
        fn test_new_with_file_path_json() {
            for test_case in FILE_TEST_CASES {
                let json_loader = data_loader_from_file(
                    LoaderKind::JSON,
                    &format!("{}{}", test_case, FILE_EXTENSIONS[0]),
                );

                #[cfg(not(target_os = "windows"))]
                let got_data = json_loader.get_data();
                #[cfg(target_os = "windows")]
                let got_data = json_loader.get_data().replace("\r\n", "\n");

                assert_eq!(
                    got_data,
                    r#"{
    "name": "John Doe",
    "age": 43,
    "address": {
        "street": "10 Downing Street",
        "city": "London"
    },
    "phones": [
        "+44 1234567",
        "+44 2345678"
    ]
}
"#
                );
            }
        }

        #[test]
        fn test_new_with_str_json() {
            for test_case in JSON_STR_TEST_CASES {
                let json_loader = data_loader_from_str(LoaderKind::JSON, test_case);
                assert_eq!(json_loader.get_data(), *test_case);
            }
        }

        #[test]
        fn test_new_with_file_path_yaml() {
            for test_case in FILE_TEST_CASES {
                let yaml_loader = data_loader_from_file(
                    LoaderKind::YAML,
                    &format!("{}{}", test_case, FILE_EXTENSIONS[1]),
                );

                #[cfg(not(target_os = "windows"))]
                let got_data = yaml_loader.get_data();
                #[cfg(target_os = "windows")]
                let got_data = yaml_loader.get_data().replace("\r\n", "\n");

                assert_eq!(
                    got_data,
                    r#"languages:
  - Ruby
  - Perl
  - Python 
websites:
  YAML: yaml.org 
  Ruby: ruby-lang.org 
  Python: python.org 
  Perl: use.perl.org
"#
                );
            }
        }

        #[test]
        fn test_new_with_str_yaml() {
            for test_case in YAML_STR_TEST_CASES {
                let yaml_loader = data_loader_from_str(LoaderKind::JSON, test_case);
                assert_eq!(yaml_loader.get_data(), *test_case);
            }
        }

        #[test]
        fn test_load() {
            let yaml_loader = data_loader_from_file(
                LoaderKind::YAML,
                &format!("{}{}", FILE_TEST_CASES[0], FILE_EXTENSIONS[1]),
            );

            let got_yaml = <DataLoader as Loader<serde_yaml::Value>>::load(&yaml_loader).unwrap();
            let expect_yaml: serde_yaml::Value =
                serde_yaml::from_str(yaml_loader.get_data()).unwrap();

            assert_eq!(got_yaml, expect_yaml);

            let json_loader = data_loader_from_file(
                LoaderKind::JSON,
                &format!("{}{}", FILE_TEST_CASES[0], FILE_EXTENSIONS[0]),
            );

            let got_json = <DataLoader as Loader<serde_json::Value>>::load(&json_loader).unwrap();
            let expect_json: serde_json::Value =
                serde_json::from_str(json_loader.get_data()).unwrap();

            assert_eq!(got_json, expect_json);
        }

        #[test]
        fn test_load_invalid() {
            let yaml_loader = data_loader_from_file(
                LoaderKind::YAML,
                &format!("{}{}", FILE_TEST_CASES[0], FILE_EXTENSIONS[1]),
            );

            match <DataLoader as Loader<serde_json::Value>>::load(&yaml_loader) {
                Ok(_) => {
                    panic!("unreachable")
                }
                Err(err) => {
                    assert_eq!(format!("{:?}", err), "Failed to String to Json Value");
                }
            }

            let json_loader = data_loader_from_file(
                LoaderKind::JSON,
                &format!("{}{}", FILE_TEST_CASES[0], FILE_EXTENSIONS[0]),
            );

            match <DataLoader as Loader<serde_yaml::Value>>::load(&json_loader) {
                Ok(_) => {
                    panic!("unreachable")
                }
                Err(err) => {
                    assert_eq!(format!("{:?}", err), "Failed to String to Yaml Value");
                }
            }
        }

        #[test]
        fn new_with_file_path_invalid() {
            match DataLoader::new_with_file_path(LoaderKind::JSON, "invalid file path") {
                Ok(_) => {
                    panic!("unreachable")
                }
                Err(err) => {
                    assert!(
                        Regex::new(r"^Failed to Load 'invalid file path'\n\nCaused by:.*")
                            .unwrap()
                            .is_match(&format!("{:?}", err))
                    );
                }
            };
        }

        #[test]
        fn test_invalid_file() {
            let invalid_json_file_path = construct_full_path("test_invalid.json").unwrap();
            let json_loader =
                DataLoader::new_with_file_path(LoaderKind::JSON, &invalid_json_file_path).unwrap();

            match <DataLoader as Loader<serde_json::Value>>::load(&json_loader) {
                Ok(_) => {
                    panic!("unreachable")
                }
                Err(err) => {
                    #[cfg(not(target_os = "windows"))]
                    let got_err = format!("{:?}", err);
                    #[cfg(target_os = "windows")]
                    let got_err = format!("{:?}", err).replace("\r\n", "\n");

                    assert_eq!(got_err, "Failed to String 'languages:\n  - Ruby\n  - Perl\n  - Python \nwebsites:\n  YAML: yaml.org \n  Ruby: ruby-lang.org \n  Python: python.org \n  Perl: use.perl.org' to Json\n\nCaused by:\n    expected value at line 1 column 1");
                }
            }

            let invalid_yaml_file_path = construct_full_path("test_invalid.yaml").unwrap();
            let yaml_loader =
                DataLoader::new_with_file_path(LoaderKind::YAML, &invalid_yaml_file_path).unwrap();

            match <DataLoader as Loader<serde_yaml::Value>>::load(&yaml_loader) {
                Ok(_) => {
                    panic!("unreachable")
                }
                Err(err) => {
                    #[cfg(not(target_os = "windows"))]
                    let got_err = format!("{:?}", err);
                    #[cfg(target_os = "windows")]
                    let got_err = format!("{:?}", err).replace("\r\n", "\n");

                    assert_eq!(got_err, "Failed to String '\"name\": \"John Doe\",\ninvalid\n' to Yaml\n\nCaused by:\n    did not find expected key at line 1 column 19, while parsing a block mapping");
                }
            }
        }
    }
}
