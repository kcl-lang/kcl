//! End-to-end check that `ExecProgramArgs::sourcemap_output` produces a
//! Source Map v3 document pointing each generated top-level key at the
//! matching `.k` source line.

use kcl_parser::ParseSession;
use kcl_runner::{ExecProgramArgs, exec_program};
use std::sync::Arc;

const FIXTURE: &str = "tests/source_map_fixture.k";

#[test]
fn end_to_end_through_runner() {
    let sess = Arc::new(ParseSession::default());
    let args = ExecProgramArgs {
        k_filename_list: vec![FIXTURE.to_string()],
        sourcemap_output: Some("out.yaml".to_string()),
        ..Default::default()
    };
    let result = exec_program(sess, &args).expect("exec");
    let yaml = &result.yaml_result;
    assert!(yaml.contains("name: alice"), "yaml was:\n{yaml}");
    assert!(yaml.contains("port: 8080"), "yaml was:\n{yaml}");
    assert!(yaml.contains("image: nginx"), "yaml was:\n{yaml}");

    let map_json = result.sourcemap.as_ref().expect("map should be set");
    let v: serde_json::Value = serde_json::from_str(map_json).expect("map is JSON");
    assert_eq!(v["version"], 3);
    assert_eq!(v["file"], "out.yaml");
    assert!(
        v["sources"][0]
            .as_str()
            .unwrap()
            .ends_with("source_map_fixture.k"),
        "sources[0] was {:?}",
        v["sources"][0]
    );

    let names: Vec<&str> = v["names"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap())
        .collect();
    assert!(names.contains(&"name"), "names: {names:?}");
    assert!(names.contains(&"port"), "names: {names:?}");
    assert!(names.contains(&"svc"), "names: {names:?}");

    // The mappings array's src_line values (1-based, one per recorded name)
    // should reach the fixture's lines 1..=3.
    let src_lines: Vec<u32> = v["mappings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["src_line"].as_u64().unwrap() as u32)
        .collect();
    let mut sorted = src_lines.clone();
    sorted.sort();
    assert!(
        sorted.starts_with(&[1, 2, 3]),
        "expected 1..=3 somewhere in src_lines, got {sorted:?}"
    );
}
