use std::{
    env,
    panic::{catch_unwind, set_hook},
};

use compiler_base_span::{FilePathMapping, SourceMap};
use kclvm_config::modfile::{get_vendor_home, KCL_PKG_PATH};

use crate::*;

use core::any::Any;

mod error_recovery;

#[macro_export]
macro_rules! parse_expr_snapshot {
    ($name:ident, $src:expr) => {
        #[test]
        fn $name() {
            insta::assert_snapshot!($crate::tests::parsing_expr_string($src));
        }
    };
}

#[macro_export]
macro_rules! parse_module_snapshot {
    ($name:ident, $src:expr) => {
        #[test]
        fn $name() {
            insta::assert_snapshot!($crate::tests::parsing_module_string($src));
        }
    };
}

pub(crate) fn parsing_expr_string(src: &str) -> String {
    let sm = SourceMap::new(FilePathMapping::empty());
    let sf = sm.new_source_file(PathBuf::from("").into(), src.to_string());
    let sess = &ParseSession::with_source_map(Arc::new(sm));

    match sf.src.as_ref() {
        Some(src_from_sf) => create_session_globals_then(|| {
            let stream = parse_token_streams(sess, src_from_sf.as_str(), new_byte_pos(0));
            let mut parser = Parser::new(sess, stream);
            let expr = parser.parse_expr();
            format!("{expr:#?}\n")
        }),
        None => "".to_string(),
    }
}

pub(crate) fn parsing_module_string(src: &str) -> String {
    let sm = SourceMap::new(FilePathMapping::empty());
    let sf = sm.new_source_file(PathBuf::from("").into(), src.to_string());
    let sess = &ParseSession::with_source_map(Arc::new(sm));

    match sf.src.as_ref() {
        Some(src_from_sf) => create_session_globals_then(|| {
            let stream = parse_token_streams(sess, src_from_sf.as_str(), new_byte_pos(0));
            let mut parser = Parser::new(sess, stream);
            let module = parser.parse_module();
            format!("{module:#?}\n")
        }),
        None => "".to_string(),
    }
}

pub fn check_result_panic_info(result: Result<(), Box<dyn Any + Send>>) {
    if let Err(e) = result {
        assert!(e.downcast::<String>().is_ok());
    };
}

const PARSE_EXPR_INVALID_TEST_CASES: &[&str] =
    &["fs1_i1re1~s", "fh==-h==-", "8_________i", "1MM", "0x00x"];

#[test]
pub fn test_parse_expr_invalid() {
    for case in PARSE_EXPR_INVALID_TEST_CASES {
        set_hook(Box::new(|_| {}));
        let result = catch_unwind(|| {
            parse_expr(case);
        });
        check_result_panic_info(result);
    }
}

const PARSE_FILE_INVALID_TEST_CASES: &[&str] = &[
    "a: int",                   // No initial value error
    "a -",                      // Invalid binary expression error
    "a?: int",                  // Invalid optional annotation error
    "a: () = 1",                // Type annotation error
    "if a not is not b: a = 1", // Logic operator error
    "if True:\n  a=1\n b=2",    // Indent error with recovery
    "a[1::::]",                 // List slice error
    "a[1 a]",                   // List index error
    "{a ++ 1}",                 // Config attribute operator error
    "func(a=1,b)",              // Call argument error
    "'${}'",                    // Empty string interpolation error
    "'${a: jso}'",              // Invalid string interpolation format spec error
];

#[test]
pub fn test_parse_file_invalid() {
    for case in PARSE_FILE_INVALID_TEST_CASES {
        let result = parse_file("test.k", Some((&case).to_string()));
        assert!(result.is_err(), "case: {case}, result {result:?}");
    }
}

pub fn test_vendor_home() {
    let vendor = &PathBuf::from(".")
        .join("testdata")
        .join("test_vendor")
        .canonicalize()
        .unwrap()
        .display()
        .to_string();
    env::set_var(KCL_PKG_PATH, vendor);
    assert_eq!(get_vendor_home(), vendor.to_string());
}

fn set_vendor_home() -> String {
    // set env vendor
    let vendor = &PathBuf::from(".")
        .join("testdata")
        .join("test_vendor")
        .canonicalize()
        .unwrap()
        .display()
        .to_string();
    env::set_var(KCL_PKG_PATH, vendor);
    debug_assert_eq!(get_vendor_home(), vendor.to_string());
    vendor.to_string()
}

#[test]
/// The testing will set environment variables,
/// so can not to execute test cases concurrently.
fn test_in_order() {
    test_import_vendor_by_external_arguments();
    println!("{:?} PASS", "test_import_vendor_by_external_arguments");
    test_import_vendor_without_vendor_home();
    println!("{:?} PASS", "test_import_vendor_without_vendor_home");
    test_import_vendor_without_kclmod();
    println!("{:?} PASS", "test_import_vendor_without_kclmod");
    test_import_vendor();
    println!("{:?} PASS", "test_import_vendor");
    test_import_vendor_with_same_internal_pkg();
    println!("{:?} PASS", "test_import_vendor_with_same_internal_pkg");
    test_import_vendor_without_kclmod_and_same_name();
    println!(
        "{:?} PASS",
        "test_import_vendor_without_kclmod_and_same_name"
    );
    test_vendor_home();
    println!("{:?} PASS", "test_vendor_home");
}

pub fn test_import_vendor() {
    let vendor = set_vendor_home();
    let sm = SourceMap::new(FilePathMapping::empty());
    let sess = Arc::new(ParseSession::with_source_map(Arc::new(sm)));

    let test_cases = vec![
        ("assign.k", vec!["__main__", "assign", "assign.assign"]),
        (
            "config_expr.k",
            vec!["__main__", "config_expr", "config_expr.config_expr_02"],
        ),
        (
            "nested_vendor.k",
            vec![
                "__main__",
                "nested_vendor",
                "nested_vendor.nested_vendor",
                "vendor_subpkg",
                "vendor_subpkg.sub.sub1",
                "vendor_subpkg.sub.sub2",
                "vendor_subpkg.sub.sub",
                "vendor_subpkg.sub",
            ],
        ),
        (
            "subpkg.k",
            vec![
                "__main__",
                "vendor_subpkg",
                "vendor_subpkg.sub.sub1",
                "vendor_subpkg.sub.sub",
                "vendor_subpkg.sub.sub2",
                "vendor_subpkg.sub",
            ],
        ),
    ];

    let dir = &PathBuf::from(".")
        .join("testdata")
        .join("import_vendor")
        .canonicalize()
        .unwrap();

    test_cases.into_iter().for_each(|(test_case_name, pkgs)| {
        let test_case_path = dir.join(test_case_name).display().to_string();
        let m = load_program(sess.clone(), &[&test_case_path], None).unwrap();
        assert_eq!(m.pkgs.len(), pkgs.len());
        m.pkgs.into_iter().for_each(|(name, modules)| {
            println!("{:?} - {:?}", test_case_name, name);
            assert!(pkgs.contains(&name.as_str()));
            for pkg in pkgs.clone() {
                if name == pkg {
                    if name == "__main__" {
                        assert_eq!(modules.len(), 1);
                        assert_eq!(modules.get(0).unwrap().filename, test_case_path);
                    } else {
                        modules.into_iter().for_each(|module| {
                            assert!(module.filename.contains(&vendor));
                        });
                    }
                    break;
                }
            }
        });
    });
}

pub fn test_import_vendor_without_kclmod() {
    let vendor = set_vendor_home();
    let sm = SourceMap::new(FilePathMapping::empty());
    let sess = Arc::new(ParseSession::with_source_map(Arc::new(sm)));

    let test_cases = vec![("import_vendor.k", vec!["__main__", "assign.assign"])];

    let dir = &PathBuf::from(".")
        .join("testdata_without_kclmod")
        .canonicalize()
        .unwrap();

    test_cases.into_iter().for_each(|(test_case_name, pkgs)| {
        let test_case_path = dir.join(test_case_name).display().to_string();
        let m = load_program(sess.clone(), &[&test_case_path], None).unwrap();
        assert_eq!(m.pkgs.len(), pkgs.len());
        m.pkgs.into_iter().for_each(|(name, modules)| {
            assert!(pkgs.contains(&name.as_str()));
            for pkg in pkgs.clone() {
                if name == pkg {
                    if name == "__main__" {
                        assert_eq!(modules.len(), 1);
                        assert_eq!(modules.get(0).unwrap().filename, test_case_path);
                    } else {
                        modules.into_iter().for_each(|module| {
                            assert!(module.filename.contains(&vendor));
                        });
                    }
                    break;
                }
            }
        });
    });
}

pub fn test_import_vendor_without_vendor_home() {
    env::set_var(KCL_PKG_PATH, "");
    let sm = SourceMap::new(FilePathMapping::empty());
    let sess = Arc::new(ParseSession::with_source_map(Arc::new(sm)));
    let dir = &PathBuf::from(".")
        .join("testdata")
        .join("import_vendor")
        .canonicalize()
        .unwrap();
    let test_case_path = dir.join("assign.k").display().to_string();
    match load_program(sess.clone(), &[&test_case_path], None) {
        Ok(_) => {
            let errors = sess.classification().0;
            let msgs = [
                "pkgpath assign not found in the program",
                "pkgpath assign.assign not found in the program",
            ];
            assert_eq!(errors.len(), msgs.len());
            for (diag, m) in errors.iter().zip(msgs.iter()) {
                assert_eq!(diag.messages[0].message, m.to_string());
            }
        }
        Err(_) => {
            panic!("Unreachable code.")
        }
    }
}

fn test_import_vendor_with_same_internal_pkg() {
    set_vendor_home();
    let sm = SourceMap::new(FilePathMapping::empty());
    let sess = Arc::new(ParseSession::with_source_map(Arc::new(sm)));
    let dir = &PathBuf::from(".")
        .join("testdata")
        .join("import_vendor")
        .canonicalize()
        .unwrap();
    let test_case_path = dir.join("same_name.k").display().to_string();
    match load_program(sess.clone(), &[&test_case_path], None) {
        Ok(_) => {
            let errors = sess.classification().0;
            let msgs = [
                "the `same_vendor` is found multiple times in the current package and vendor package"
            ];
            assert_eq!(errors.len(), msgs.len());
            for (diag, m) in errors.iter().zip(msgs.iter()) {
                assert_eq!(diag.messages[0].message, m.to_string());
            }
        }
        Err(_) => {
            panic!("Unreachable code.")
        }
    }
}

fn test_import_vendor_without_kclmod_and_same_name() {
    set_vendor_home();
    let sm = SourceMap::new(FilePathMapping::empty());
    let sess = Arc::new(ParseSession::with_source_map(Arc::new(sm)));
    let dir = &PathBuf::from(".")
        .join("testdata_without_kclmod")
        .join("same_name")
        .canonicalize()
        .unwrap();
    let test_case_path = dir.join("assign.k").display().to_string();
    match load_program(sess.clone(), &[&test_case_path], None) {
        Ok(_) => {
            let errors = sess.classification().0;
            let msgs = [
                "the `assign` is found multiple times in the current package and vendor package",
            ];
            assert_eq!(errors.len(), msgs.len());
            for (diag, m) in errors.iter().zip(msgs.iter()) {
                assert_eq!(diag.messages[0].message, m.to_string());
            }
        }
        Err(_) => {
            panic!("Unreachable code.")
        }
    }
}

fn test_import_vendor_by_external_arguments() {
    let vendor = set_vendor_home();
    let sm = SourceMap::new(FilePathMapping::empty());
    let sess = Arc::new(ParseSession::with_source_map(Arc::new(sm)));

    let external_dir = &PathBuf::from(".")
        .join("testdata")
        .join("test_vendor")
        .canonicalize()
        .unwrap();

    let test_cases = vec![
        (
            "import_by_external_assign.k",
            "assign",
            vec!["__main__", "assign"],
        ),
        (
            "import_by_external_config_expr.k",
            "config_expr",
            vec!["__main__", "config_expr"],
        ),
        (
            "import_by_external_nested_vendor.k",
            "nested_vendor",
            vec![
                "__main__",
                "nested_vendor",
                "vendor_subpkg",
                "vendor_subpkg.sub.sub2",
                "vendor_subpkg.sub.sub1",
                "vendor_subpkg.sub.sub",
                "vendor_subpkg.sub",
            ],
        ),
        (
            "import_by_external_vendor_subpkg.k",
            "vendor_subpkg",
            vec![
                "__main__",
                "vendor_subpkg",
                "vendor_subpkg.sub.sub1",
                "vendor_subpkg.sub.sub2",
                "vendor_subpkg.sub.sub",
                "vendor_subpkg.sub",
            ],
        ),
    ];

    let dir = &PathBuf::from(".")
        .join("testdata_without_kclmod")
        .canonicalize()
        .unwrap();

    test_cases
        .into_iter()
        .for_each(|(test_case_name, dep_name, pkgs)| {
            let mut opts = LoadProgramOptions::default();
            opts.package_maps.insert(
                dep_name.to_string(),
                external_dir.join(dep_name).display().to_string(),
            );
            let test_case_path = dir.join(test_case_name).display().to_string();
            let m = load_program(sess.clone(), &[&test_case_path], None).unwrap();

            assert_eq!(m.pkgs.len(), pkgs.len());
            m.pkgs.into_iter().for_each(|(name, modules)| {
                assert!(pkgs.contains(&name.as_str()));
                for pkg in pkgs.clone() {
                    if name == pkg {
                        if name == "__main__" {
                            assert_eq!(modules.len(), 1);
                            assert_eq!(modules.get(0).unwrap().filename, test_case_path);
                        } else {
                            modules.into_iter().for_each(|module| {
                                assert!(module.filename.contains(&vendor));
                            });
                        }
                        break;
                    }
                }
            });
        });
}
