use std::path::PathBuf;

use kcl_parser::ParseSessionRef;
use kcl_runner::{ExecProgramArgs, exec_program};

use super::bundle;

fn fixture(rel: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src");
    path.push("bundle");
    path.push("test_data");
    path.push(rel);
    path
}

fn run_yaml(files: Vec<String>) -> String {
    let args = ExecProgramArgs {
        k_filename_list: files,
        ..Default::default()
    };
    exec_program(ParseSessionRef::default(), &args)
        .unwrap()
        .yaml_result
}

#[test]
fn test_bundle_rewrites_and_inlines() {
    let entry = fixture("simple/main.k");
    let bundled = bundle(&[entry.to_str().unwrap()], None).unwrap();
    // Imports of in-tree packages are inlined, builtin imports are kept.
    assert!(bundled.contains("import math"));
    assert!(!bundled.contains("import .sub.orphan"));
    assert!(!bundled.contains("import .types"));
    // Symbols of imported packages are renamed with a package prefix. The
    // leading `_` keeps the inlined values out of the output.
    assert!(bundled.contains("_sub__y = math.floor(2.5)"));
    assert!(bundled.contains("schema _types__Server:"));
    assert!(bundled.contains("schema _types_nested__Box:"));
    // References through import aliases (incl. `as` names) are rewritten.
    assert!(bundled.contains("value = _sub__y + math.floor(1.5)"));
    assert!(bundled.contains("server = _types__Server"));
    assert!(bundled.contains("box = _types_nested__Box"));
}

#[test]
fn test_bundle_is_equivalent_to_the_original_program() {
    for rel in ["simple/main.k", "advanced/main.k"] {
        let entry = fixture(rel);
        let entry = entry.to_str().unwrap().to_string();
        let original = run_yaml(vec![entry.clone()]);
        let bundled = bundle(&[entry.as_str()], None).unwrap();
        let out = std::env::temp_dir().join("kcl_bundle_test.k");
        std::fs::write(&out, &bundled).unwrap();
        let bundled = run_yaml(vec![out.to_str().unwrap().to_string()]);
        assert_eq!(original, bundled, "bundling {} changed the output", rel);
    }
}
