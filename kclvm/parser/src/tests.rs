use std::{
    env,
    panic::{catch_unwind, set_hook},
};

use compiler_base_span::{FilePathMapping, SourceMap};
use kclvm_config::modfile::{get_vendor_home, KCL_PKG_PATH};

use crate::*;

use core::any::Any;

mod ast;
mod error_recovery;
mod expr;
mod file;
mod types;

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

#[macro_export]
macro_rules! parse_type_snapshot {
    ($name:ident, $src:expr) => {
        #[test]
        fn $name() {
            insta::assert_snapshot!($crate::tests::parsing_type_string($src));
        }
    };
}

#[macro_export]
macro_rules! parse_type_node_snapshot {
    ($name:ident, $src:expr) => {
        #[test]
        fn $name() {
            insta::assert_snapshot!($crate::tests::parsing_type_node_string($src));
        }
    };
}

#[macro_export]
macro_rules! parse_file_ast_json_snapshot {
    ($name:ident, $filename:expr, $src:expr) => {
        #[test]
        fn $name() {
            insta::assert_snapshot!($crate::tests::parsing_file_ast_json($filename, $src));
        }
    };
}

#[macro_export]
macro_rules! parse_file_snapshot {
    ($name:ident, $filename:expr) => {
        #[test]
        fn $name() {
            insta::assert_snapshot!($crate::tests::parsing_file_string($filename));
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

pub(crate) fn parsing_type_string(src: &str) -> String {
    let sm = SourceMap::new(FilePathMapping::empty());
    sm.new_source_file(PathBuf::from("").into(), src.to_string());
    let sess = &ParseSession::with_source_map(Arc::new(sm));

    create_session_globals_then(|| {
        let stream = parse_token_streams(sess, src, new_byte_pos(0));
        let mut parser = Parser::new(sess, stream);
        let typ = parser.parse_type_annotation();
        format!("{typ:#?}\n")
    })
}

pub(crate) fn parsing_type_node_string(src: &str) -> String {
    let sm = SourceMap::new(FilePathMapping::empty());
    sm.new_source_file(PathBuf::from("").into(), src.to_string());
    let sess = &ParseSession::with_source_map(Arc::new(sm));

    create_session_globals_then(|| {
        let stream = parse_token_streams(sess, src, new_byte_pos(0));
        let mut parser = Parser::new(sess, stream);
        let typ = parser.parse_type_annotation();
        typ.node.to_string()
    })
}

pub(crate) fn parsing_file_ast_json(filename: &str, src: &str) -> String {
    let m = crate::parse_file_with_global_session(
        Arc::new(ParseSession::default()),
        filename,
        Some(src.into()),
    )
    .unwrap();
    serde_json::ser::to_string_pretty(&m).unwrap()
}

pub(crate) fn parsing_file_string(filename: &str) -> String {
    let code = std::fs::read_to_string(filename).unwrap();
    let m = crate::parse_file(filename.trim_start_matches("testdata/"), Some(code))
        .expect(filename)
        .module;
    serde_json::ser::to_string_pretty(&m).unwrap()
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
        let result = parse_file_force_errors("test.k", Some((&case).to_string()));
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
    test_pkg_not_found_suggestion();
    println!("{:?} PASS", "test_pkg_not_found_suggestion");
}

pub fn test_import_vendor() {
    let module_cache = KCLModuleCache::default();
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

    let test_fn =
        |test_case_name: &&str, pkgs: &Vec<&str>, module_cache: Option<KCLModuleCache>| {
            let test_case_path = dir.join(test_case_name).display().to_string();
            let m = load_program(sess.clone(), &[&test_case_path], None, module_cache)
                .unwrap()
                .program;
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
        };

    test_cases
        .iter()
        .for_each(|(test_case_name, pkgs)| test_fn(test_case_name, pkgs, None));

    test_cases.iter().for_each(|(test_case_name, pkgs)| {
        test_fn(test_case_name, pkgs, Some(module_cache.clone()))
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
        let m = load_program(sess.clone(), &[&test_case_path], None, None)
            .unwrap()
            .program;
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
    match load_program(sess.clone(), &[&test_case_path], None, None) {
        Ok(_) => {
            let errors = sess.classification().0;
            let msgs = [
                "pkgpath assign not found in the program",
                "try 'kcl mod add assign' to download the package not found",
                "find more package on 'https://artifacthub.io'",
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

    match load_program(
        sess.clone(),
        &[&test_case_path],
        None,
        Some(KCLModuleCache::default()),
    ) {
        Ok(_) => {
            let errors = sess.classification().0;
            let msgs = [
                "pkgpath assign not found in the program",
                "try 'kcl mod add assign' to download the package not found",
                "find more package on 'https://artifacthub.io'",
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
    match load_program(sess.clone(), &[&test_case_path], None, None) {
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
    match load_program(
        sess.clone(),
        &[&test_case_path],
        None,
        Some(KCLModuleCache::default()),
    ) {
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
    match load_program(sess.clone(), &[&test_case_path], None, None) {
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

    match load_program(
        sess.clone(),
        &[&test_case_path],
        None,
        Some(KCLModuleCache::default()),
    ) {
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
    let module_cache = KCLModuleCache::default();
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

    let test_fn = |test_case_name: &&str,
                   dep_name: &&str,
                   pkgs: &Vec<&str>,
                   module_cache: Option<KCLModuleCache>| {
        let mut opts = LoadProgramOptions::default();
        opts.package_maps.insert(
            dep_name.to_string(),
            external_dir.join(dep_name).display().to_string(),
        );
        let test_case_path = dir.join(test_case_name).display().to_string();
        let m = load_program(sess.clone(), &[&test_case_path], None, module_cache)
            .unwrap()
            .program;
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
    };

    test_cases
        .iter()
        .for_each(|(test_case_name, dep_name, pkgs)| test_fn(test_case_name, dep_name, pkgs, None));

    test_cases
        .iter()
        .for_each(|(test_case_name, dep_name, pkgs)| {
            test_fn(test_case_name, dep_name, pkgs, Some(module_cache.clone()))
        });
}

#[test]
fn test_get_compile_entries_from_paths() {
    let testpath = PathBuf::from("./src/testdata/multimods")
        .canonicalize()
        .unwrap();

    // [`kcl1_path`] is a normal path of the package [`kcl1`] root directory.
    // It looks like `/xxx/xxx/xxx`.
    let kcl1_path = testpath.join("kcl1");

    // [`kcl2_path`] is a mod relative path of the packege [`kcl2`] root directory.
    // It looks like `${kcl2:KCL_MOD}/xxx/xxx`
    let kcl2_path = PathBuf::from("${kcl2:KCL_MOD}/main.k");

    // [`kcl3_path`] is a mod relative path of the [`__main__`] packege.
    let kcl3_path = PathBuf::from("${KCL_MOD}/main.k");

    // [`package_maps`] is a map to show the real path of the mod relative path [`kcl2`].
    let mut opts = LoadProgramOptions::default();
    opts.package_maps.insert(
        "kcl2".to_string(),
        testpath.join("kcl2").to_str().unwrap().to_string(),
    );

    // [`get_compile_entries_from_paths`] will return the map of package name to package root real path.
    let entries = get_compile_entries_from_paths(
        &[
            kcl1_path.to_str().unwrap().to_string(),
            kcl2_path.display().to_string(),
            kcl3_path.display().to_string(),
        ],
        &opts,
    )
    .unwrap();

    assert_eq!(entries.len(), 3);

    assert_eq!(entries.get_nth_entry(0).unwrap().name(), "__main__");
    assert_eq!(
        PathBuf::from(entries.get_nth_entry(0).unwrap().path())
            .canonicalize()
            .unwrap()
            .display()
            .to_string(),
        kcl1_path.canonicalize().unwrap().to_str().unwrap()
    );

    assert_eq!(entries.get_nth_entry(1).unwrap().name(), "kcl2");
    assert_eq!(
        PathBuf::from(entries.get_nth_entry(1).unwrap().path())
            .canonicalize()
            .unwrap()
            .display()
            .to_string(),
        testpath
            .join("kcl2")
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
    );

    assert_eq!(entries.get_nth_entry(2).unwrap().name(), "__main__");
    assert_eq!(
        PathBuf::from(entries.get_nth_entry(2).unwrap().path())
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap(),
        kcl1_path.canonicalize().unwrap().to_str().unwrap()
    );
}

#[test]
fn test_dir_with_k_code_list() {
    let sm = SourceMap::new(FilePathMapping::empty());
    let sess = Arc::new(ParseSession::with_source_map(Arc::new(sm)));
    let testpath = PathBuf::from("./src/testdata/test_k_code_list")
        .canonicalize()
        .unwrap();

    let mut opts = LoadProgramOptions::default();
    opts.k_code_list = vec!["test_code = 1".to_string()];

    match load_program(
        sess.clone(),
        &[&testpath.display().to_string()],
        Some(opts.clone()),
        None,
    ) {
        Ok(_) => panic!("unreachable code"),
        Err(err) => assert!(err.to_string().contains("Invalid code list")),
    }

    match load_program(
        sess.clone(),
        &[&testpath.display().to_string()],
        Some(opts),
        Some(KCLModuleCache::default()),
    ) {
        Ok(_) => panic!("unreachable code"),
        Err(err) => assert!(err.to_string().contains("Invalid code list")),
    }
}

pub fn test_pkg_not_found_suggestion() {
    let sm = SourceMap::new(FilePathMapping::empty());
    let sess = Arc::new(ParseSession::with_source_map(Arc::new(sm)));
    let dir = &PathBuf::from("./src/testdata/pkg_not_found")
        .canonicalize()
        .unwrap();
    let test_case_path = dir.join("suggestions.k").display().to_string();
    match load_program(sess.clone(), &[&test_case_path], None, None) {
        Ok(_) => {
            let errors = sess.classification().0;
            let msgs = [
                "pkgpath k9s not found in the program",
                "try 'kcl mod add k9s' to download the package not found",
                "find more package on 'https://artifacthub.io'",
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
