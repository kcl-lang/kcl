use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::util::loader::LoaderKind;

const CARGO_DIR: &str = env!("CARGO_MANIFEST_DIR");
const REL_PATH: &str = "src/vet/test_datas";
const NO_SCHEMA_NAME_PATH: &str = "no_schema_name";

const TEST_CASES: &'static [&'static str] = &[
    "test.k",
    "simple.k",
    "plain_value.k",
    "list.k",
    "complex.k",
    "only_with_null",
    "only_with_bool",
    "only_with_float",
];

const SCHEMA_NAMES: &'static [&'static str] = &[
    "test",
    "simple",
    "plain_value",
    "list",
    "complex",
    "only_with_null",
    "only_with_bool",
    "only_with_float",
];

const FILE_EXTENSIONS: &'static [&'static str] = &["json", "yaml", "ast.json", "ast.yaml", "k"];

const LOADER_KIND: [&LoaderKind; 2] = [&LoaderKind::JSON, &LoaderKind::YAML];

const INVALID_FILE_RESULT: &'static [&'static str] = &[
"Failed to Load JSON\n\nCaused by:\n    0: Failed to String 'languages:\n         - Ruby\n       ' to Json\n    1: expected value at line 1 column 1", 
"Failed to Load YAML\n\nCaused by:\n    0: Failed to String '{\n           \"name\": \"John Doe\",\n               \"city\": \"London\"\n       invalid\n       \n       ' to Yaml\n    1: did not find expected ',' or '}' at line 4 column 1, while parsing a flow mapping"
];

fn construct_full_path(path: &str) -> Result<String> {
    let mut cargo_file_path = PathBuf::from(CARGO_DIR);
    cargo_file_path.push(REL_PATH);
    cargo_file_path.push(path);
    Ok(cargo_file_path
        .to_str()
        .with_context(|| format!("No such file or directory '{}'", path))?
        .to_string())
}

mod test_expr_builder {
    use crate::{
        util::loader::LoaderKind,
        vet::{
            expr_builder::ExprBuilder,
            tests::{
                construct_full_path, FILE_EXTENSIONS, INVALID_FILE_RESULT, LOADER_KIND,
                NO_SCHEMA_NAME_PATH, SCHEMA_NAMES, TEST_CASES,
            },
        },
    };
    use std::{
        fs::{self, File},
        panic,
    };

    #[test]
    fn test_build_with_json_no_schema_name() {
        for test_name in TEST_CASES {
            let file_path = construct_full_path(&format!(
                "{}/{}.{}",
                FILE_EXTENSIONS[0], test_name, FILE_EXTENSIONS[0]
            ))
            .unwrap();
            let expr_builder =
                ExprBuilder::new_with_file_path(*LOADER_KIND[0], file_path.clone()).unwrap();
            let expr_ast = expr_builder.build(None).unwrap();
            let got_ast_json = serde_json::to_value(&expr_ast).unwrap();

            let expect_file_path = construct_full_path(&format!(
                "{}/{}/{}.{}",
                FILE_EXTENSIONS[0], NO_SCHEMA_NAME_PATH, test_name, FILE_EXTENSIONS[2]
            ))
            .unwrap();
            let f = File::open(expect_file_path.clone()).unwrap();
            let expect_ast_json: serde_json::Value = serde_json::from_reader(f).unwrap();
            assert_eq!(expect_ast_json, got_ast_json)
        }
    }

    #[test]
    fn test_build_with_yaml_no_schema_name() {
        for test_name in TEST_CASES {
            let file_path = construct_full_path(&format!(
                "{}/{}.{}",
                FILE_EXTENSIONS[1], test_name, FILE_EXTENSIONS[1]
            ))
            .unwrap();
            let expr_builder =
                ExprBuilder::new_with_file_path(*LOADER_KIND[1], file_path.clone()).unwrap();
            let expr_ast = expr_builder.build(None).unwrap();
            let got_ast_yaml = serde_yaml::to_value(&expr_ast).unwrap();

            let expect_file_path = construct_full_path(&format!(
                "{}/{}/{}.{}",
                FILE_EXTENSIONS[1], NO_SCHEMA_NAME_PATH, test_name, FILE_EXTENSIONS[3]
            ))
            .unwrap();
            let f = File::open(expect_file_path.clone()).unwrap();
            let expect_ast_yaml: serde_yaml::Value = serde_yaml::from_reader(f).unwrap();
            if expect_ast_yaml != got_ast_yaml {
                serde_yaml::to_writer(std::io::stdout(), &got_ast_yaml).unwrap();
            }
            assert_eq!(expect_ast_yaml, got_ast_yaml)
        }
    }

    #[test]
    /// Test `expr_builder.build()` with input json files.
    fn test_build_json_with_filepath() {
        for i in 0..TEST_CASES.len() {
            let file_path =
                construct_full_path(&format!("{1}/{0}.{1}", TEST_CASES[i], FILE_EXTENSIONS[0]))
                    .unwrap();
            let expr_builder =
                ExprBuilder::new_with_file_path(LOADER_KIND[0].clone(), file_path).unwrap();
            let expr_ast = expr_builder
                .build(Some(SCHEMA_NAMES[i].to_string()))
                .unwrap();
            let got_ast_json = serde_json::to_value(&expr_ast).unwrap();

            let expect_file_path = construct_full_path(&format!(
                "{}/{}.{}",
                FILE_EXTENSIONS[0], TEST_CASES[i], FILE_EXTENSIONS[2]
            ))
            .unwrap();
            let f = File::open(expect_file_path.clone()).unwrap();
            let expect_ast_json: serde_json::Value = serde_json::from_reader(f).unwrap();
            assert_eq!(expect_ast_json, got_ast_json)
        }
    }

    #[test]
    /// Test `expr_builder.build()` with input json files.
    fn test_build_json_with_str() {
        for i in 0..TEST_CASES.len() {
            let file_path =
                construct_full_path(&format!("{1}/{0}.{1}", TEST_CASES[i], FILE_EXTENSIONS[0]))
                    .unwrap();

            let content = fs::read_to_string(file_path).unwrap();

            let expr_builder = ExprBuilder::new_with_str(LOADER_KIND[0].clone(), content).unwrap();
            let expr_ast = expr_builder
                .build(Some(SCHEMA_NAMES[i].to_string()))
                .unwrap();
            let got_ast_json = serde_json::to_value(&expr_ast).unwrap();

            let expect_file_path = construct_full_path(&format!(
                "{}/{}.{}",
                FILE_EXTENSIONS[0], TEST_CASES[i], FILE_EXTENSIONS[2]
            ))
            .unwrap();
            let f = File::open(expect_file_path.clone()).unwrap();
            let expect_ast_json: serde_json::Value = serde_json::from_reader(f).unwrap();
            assert_eq!(expect_ast_json, got_ast_json)
        }
    }

    #[test]
    /// Test `expr_builder.build()` with input yaml files.
    fn test_build_yaml() {
        for i in 0..TEST_CASES.len() {
            let file_path =
                construct_full_path(&format!("{1}/{0}.{1}", TEST_CASES[i], FILE_EXTENSIONS[1]))
                    .unwrap();
            let expr_builder =
                ExprBuilder::new_with_file_path(LOADER_KIND[1].clone(), file_path).unwrap();
            let expr_ast = expr_builder
                .build(Some(SCHEMA_NAMES[i].to_string()))
                .unwrap();
            let got_ast_yaml = serde_yaml::to_value(&expr_ast).unwrap();

            let expect_file_path = construct_full_path(&format!(
                "{}/{}.{}",
                FILE_EXTENSIONS[1], TEST_CASES[i], FILE_EXTENSIONS[3]
            ))
            .unwrap();
            let f = File::open(expect_file_path.clone()).unwrap();
            let expect_ast_yaml: serde_yaml::Value = serde_yaml::from_reader(f).unwrap();
            assert_eq!(expect_ast_yaml, got_ast_yaml)
        }
    }

    #[test]
    /// Test `expr_builder.build()` with input invalid json/yaml files.
    fn test_build_with_invalid() {
        for i in 0..2 {
            let file_path = construct_full_path(&format!(
                "invalid/{}.{}",
                "test_invalid", FILE_EXTENSIONS[i]
            ))
            .unwrap();
            let expr_builder = ExprBuilder::new_with_file_path(*LOADER_KIND[i], file_path).unwrap();
            match expr_builder.build(None) {
                Ok(_) => {
                    panic!("This test case should be failed.")
                }
                Err(err) => {
                    assert_eq!(format!("{:?}", err), INVALID_FILE_RESULT[i]);
                }
            };
        }
    }

    #[test]
    /// Test `expr_builder.build()` with files that do not exist.
    fn test_build_with_noexist_file() {
        for i in 0..2 {
            let file_path = construct_full_path(&format!(
                "json/{}.{}",
                "test_json_not_exist", FILE_EXTENSIONS[i]
            ))
            .unwrap();
            match ExprBuilder::new_with_file_path(*LOADER_KIND[i], file_path.clone()) {
                Ok(_) => {
                    panic!("This test case should be failed.")
                }
                Err(err) => {
                    assert_eq!(
                        format!("{:?}", err), 
                        format!("Failed to Load '{0}'\n\nCaused by:\n    0: Failed to Load '{0}'\n    1: No such file or directory (os error 2)", file_path)
                    )
                }
            };
        }
    }

    #[test]
    /// Test `expr_builder.build()` with yaml files and json data loader.
    fn test_build_with_yaml_file_with_json_kind() {
        let file_path = construct_full_path(&format!("yaml/{}", "test.k.yaml")).unwrap();
        let expr_builder =
            ExprBuilder::new_with_file_path(LoaderKind::JSON, file_path.clone()).unwrap();

        match expr_builder.build(None) {
            Ok(_) => {
                panic!("This test case should be failed.")
            }
            Err(err) => {
                assert_eq!(
                    format!("{:?}", err), 
                    "Failed to Load JSON\n\nCaused by:\n    0: Failed to String 'languages:\n         - Ruby\n         - Perl\n         - Python \n       websites:\n         YAML: yaml.org \n         Ruby: ruby-lang.org \n         Python: python.org \n         Perl: use.perl.org\n       ' to Json\n    1: expected value at line 1 column 1"
                )
            }
        }
    }

    #[test]
    fn test_unsupported_u64_json() {
        // unsupported u64 json
        let file_path = construct_full_path("invalid/unsupported/json_with_u64.json").unwrap();
        let expr_builder =
            ExprBuilder::new_with_file_path(*LOADER_KIND[0], file_path.clone()).unwrap();
        match expr_builder.build(None) {
            Ok(_) => {
                panic!("unreachable")
            }
            Err(err) => {
                assert_eq!(format!("{:?}", err), "Failed to Load JSON\n\nCaused by:\n    0: Failed to Load Validated File\n    1: Failed to Load Validated File, Unsupported Unsigned 64");
            }
        };
    }

    #[test]
    fn test_unsupported_u64_yaml() {
        // unsupported u64 yaml
        let file_path = construct_full_path("invalid/unsupported/yaml_with_u64.yaml").unwrap();
        let expr_builder =
            ExprBuilder::new_with_file_path(*LOADER_KIND[1], file_path.clone()).unwrap();
        match expr_builder.build(None) {
            Ok(_) => {
                panic!("unreachable")
            }
            Err(err) => {
                assert_eq!(format!("{:?}", err), "Failed to Load YAML\n\nCaused by:\n    0: Failed to Load Validated File\n    1: Failed to Load Validated File, Unsupported Unsigned 64");
            }
        };
    }

    #[test]
    fn test_unsupported_yaml_with_tag() {
        // unsupported yaml with tag
        let file_path = construct_full_path("invalid/unsupported/yaml_with_tag.yaml").unwrap();
        let expr_builder =
            ExprBuilder::new_with_file_path(*LOADER_KIND[1], file_path.clone()).unwrap();
        match expr_builder.build(None) {
            Ok(_) => {
                panic!("unreachable")
            }
            Err(err) => {
                assert_eq!(format!("{:?}", err), "Failed to Load YAML\n\nCaused by:\n    Failed to Load Validated File, Unsupported Yaml Tagged.");
            }
        };
    }
}

mod test_validater {
    use std::{fs, panic};

    use crate::{
        util::loader::LoaderKind,
        vet::validator::{validate, ValidateOption},
    };

    use super::{construct_full_path, LOADER_KIND};

    const KCL_TEST_CASES: &'static [&'static str] =
        &["test.k", "simple.k", "list.k", "plain_value.k", "complex.k"];
    const VALIDATED_FILE_TYPE: &'static [&'static str] = &["json", "yaml"];

    #[test]
    fn test_validator() {
        test_validate();
        test_invalid_validate();
        test_validate_with_invalid_kcl_path();
        test_validate_with_invalid_file_path();
        test_validate_with_invalid_file_type();
    }

    fn test_validate() {
        for (i, file_suffix) in VALIDATED_FILE_TYPE.iter().enumerate() {
            for case in KCL_TEST_CASES {
                let validated_file_path =
                    construct_full_path(&format!("{}/{}.{}", "validate_cases", case, file_suffix))
                        .unwrap();

                let kcl_file_path =
                    construct_full_path(&format!("{}/{}", "validate_cases", case)).unwrap();

                let opt = ValidateOption::new(
                    None,
                    "value".to_string(),
                    validated_file_path.clone(),
                    *LOADER_KIND[i],
                    Some(kcl_file_path.to_string()),
                    None,
                );

                match validate(opt) {
                    Ok(res) => assert!(res),
                    Err(_) => panic!("Unreachable"),
                }
            }
        }
    }

    fn test_invalid_validate() {
        let prev_hook = std::panic::take_hook();
        // disable print panic info
        std::panic::set_hook(Box::new(|_| {}));

        for (i, file_suffix) in VALIDATED_FILE_TYPE.iter().enumerate() {
            for case in KCL_TEST_CASES {
                let validated_file_path = construct_full_path(&format!(
                    "{}/{}.{}",
                    "invalid_validate_cases", case, file_suffix
                ))
                .unwrap();

                let kcl_code = fs::read_to_string(
                    construct_full_path(&format!("{}/{}", "invalid_validate_cases", case)).unwrap(),
                )
                .expect("Something went wrong reading the file");

                let expected_err_msg = fs::read_to_string(
                    construct_full_path(&format!(
                        "{}/{}.{}",
                        "invalid_validate_cases", case, "stderr.json"
                    ))
                    .unwrap(),
                )
                .expect("Something went wrong reading the file");

                let opt = ValidateOption::new(
                    None,
                    "value".to_string(),
                    validated_file_path.clone(),
                    *LOADER_KIND[i],
                    None,
                    Some(kcl_code),
                );

                let result = panic::catch_unwind(|| validate(opt));

                let expect: serde_json::Value = serde_json::from_str(&expected_err_msg).unwrap();
                match result {
                    Ok(result) => match result {
                        Ok(_) => {
                            panic!("Unreachable.")
                        }
                        Err(err) => {
                            let got: serde_json::Value = serde_json::from_str(&err).unwrap();
                            assert_eq!(got, expect);
                        }
                    },
                    Err(panic_err) => {
                        if let Some(result) = panic_err.downcast_ref::<String>() {
                            let got: serde_json::Value = serde_json::from_str(&result).unwrap();
                            assert_eq!(got, expect);
                        } else {
                            panic!("Unreachable.")
                        };
                    }
                }
            }
        }
        std::panic::set_hook(prev_hook);
    }

    fn test_validate_with_invalid_kcl_path() {
        let opt = ValidateOption::new(
            None,
            "value".to_string(),
            "The validated file path is invalid".to_string(),
            LoaderKind::JSON,
            None,
            None,
        );

        match validate(opt) {
            Ok(_) => {
                panic!("unreachable")
            }
            Err(err) => {
                assert_eq!(err,
                "Failed to load KCL file 'validationTempKCLCode.k'. Because 'No such file or directory (os error 2)'")
            }
        }
    }

    fn test_validate_with_invalid_file_path() {
        let kcl_code = fs::read_to_string(
            construct_full_path(&format!("{}/{}", "validate_cases", "test.k")).unwrap(),
        )
        .expect("Something went wrong reading the file");

        let opt = ValidateOption::new(
            None,
            "value".to_string(),
            "The validated file path is invalid".to_string(),
            LoaderKind::JSON,
            None,
            Some(kcl_code),
        );

        match validate(opt) {
            Ok(_) => {
                panic!("unreachable")
            }
            Err(err) => {
                assert_eq!(err, "Failed to load validated file.")
            }
        }
    }

    fn test_validate_with_invalid_file_type() {
        let kcl_code = fs::read_to_string(
            construct_full_path(&format!("{}/{}", "validate_cases", "test.k")).unwrap(),
        )
        .expect("Something went wrong reading the file");

        let validated_file_path =
            construct_full_path(&format!("{}/{}", "validate_cases", "test.k.yaml")).unwrap();

        let opt = ValidateOption::new(
            None,
            "value".to_string(),
            validated_file_path,
            LoaderKind::JSON,
            None,
            Some(kcl_code),
        );

        match validate(opt) {
            Ok(_) => {
                panic!("unreachable")
            }
            Err(err) => {
                assert_eq!(err, "Failed to load validated file.")
            }
        }
    }
}
