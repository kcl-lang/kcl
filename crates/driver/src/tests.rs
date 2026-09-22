use std::panic;
use std::path::PathBuf;

use kcl_config::settings::KeyValuePair;
use kcl_utils::path::PathPrefix;

use crate::arguments::parse_key_value_pair;
use crate::toolchain::NativeToolchain;
use crate::toolchain::Toolchain;
use crate::{
    CompileUnitPath, WorkSpaceKind, get_pkg_list, lookup_compile_unit_path,
    lookup_compile_unit_path_bounded, lookup_compile_workspace, lookup_compile_workspace_bounded,
    lookup_the_nearest_file_dir, toolchain,
};

fn lookup_walkup_dir(rel: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("test_data")
        .join("lookup_walkup")
        .join(rel);
    // Strip the Windows `\\?\` prefix that `canonicalize` adds so the
    // returned path matches the form produced by
    // `lookup_compile_unit_path_internal` and `lookup_workspace_bounded`.
    PathBuf::from(dir.canonicalize().unwrap().adjust_canonicalization())
}

#[test]
fn test_parse_key_value_pair() {
    let cases = [
        (
            "k=v",
            KeyValuePair {
                key: "k".to_string(),
                value: "\"v\"".into(),
            },
        ),
        (
            "k=1",
            KeyValuePair {
                key: "k".to_string(),
                value: "1".into(),
            },
        ),
        (
            "k=None",
            KeyValuePair {
                key: "k".to_string(),
                value: "null".into(),
            },
        ),
        (
            "k=True",
            KeyValuePair {
                key: "k".to_string(),
                value: "true".into(),
            },
        ),
        (
            "k=true",
            KeyValuePair {
                key: "k".to_string(),
                value: "true".into(),
            },
        ),
        (
            "k={\"key\": \"value\"}",
            KeyValuePair {
                key: "k".to_string(),
                value: "{\"key\": \"value\"}".into(),
            },
        ),
        (
            "k=[1, 2, 3]",
            KeyValuePair {
                key: "k".to_string(),
                value: "[1, 2, 3]".into(),
            },
        ),
        // Test scientific notation - should be treated as string
        (
            "k=12e1",
            KeyValuePair {
                key: "k".to_string(),
                value: "\"12e1\"".into(),
            },
        ),
        (
            "k=1.5e-3",
            KeyValuePair {
                key: "k".to_string(),
                value: "\"1.5e-3\"".into(),
            },
        ),
        (
            "k=2E10",
            KeyValuePair {
                key: "k".to_string(),
                value: "\"2E10\"".into(),
            },
        ),
    ];
    for (value, pair) in cases {
        let result = parse_key_value_pair(value).unwrap();
        assert_eq!(result.key, pair.key);
        assert_eq!(result.value, pair.value);
    }
}

#[test]
fn test_parse_key_value_pair_fail() {
    let cases = ["=v", "k=", "="];
    for case in cases {
        assert!(parse_key_value_pair(case).is_err());
    }
}

#[test]
fn test_lookup_the_nearest_file_dir() {
    let path = PathBuf::from(".")
        .join("src")
        .join("test_data")
        .join("kpm_metadata");
    let result = lookup_the_nearest_file_dir(path.clone(), "kcl.mod");
    assert!(result.is_some());
    assert_eq!(
        result.unwrap().display().to_string(),
        path.canonicalize().unwrap().display().to_string()
    );

    let main_path = path.join("subdir").join("main.k");
    let result = lookup_the_nearest_file_dir(main_path, "kcl.mod");
    assert!(result.is_some());
    assert_eq!(
        result.unwrap().display().to_string(),
        path.canonicalize().unwrap().display().to_string()
    );
}

#[test]
fn test_fetch_metadata_invalid() {
    let result = panic::catch_unwind(|| {
        let tool = toolchain::default();
        let result = tool.fetch_metadata("invalid_path".to_string().into());
        match result {
            Ok(_) => {
                panic!("The method should not return Ok")
            }
            Err(_) => {
                println!("return with an error.")
            }
        }
    });

    match result {
        Ok(_) => println!("no panic"),
        Err(e) => panic!("The method should not panic forever.: {:?}", e),
    }
}

#[test]
fn test_native_fetch_metadata_invalid() {
    let result = panic::catch_unwind(|| {
        let tool = NativeToolchain::default();
        let result = tool.fetch_metadata("invalid_path".to_string().into());
        match result {
            Ok(_) => {
                panic!("The method should not return Ok")
            }
            Err(_) => {
                println!("return with an error.")
            }
        }
    });

    match result {
        Ok(_) => println!("no panic"),
        Err(e) => panic!("The method should not panic forever.: {:?}", e),
    }
}

#[test]
fn test_get_pkg_list() {
    // Single package: the binding fix in `get_pkg_list` should resolve
    // the relative input to an absolute directory and return it as-is.
    let single = get_pkg_list("./src/test_data/pkg_list/").unwrap();
    assert_eq!(single.len(), 1);
    assert!(
        PathBuf::from(&single[0]).is_absolute(),
        "expected absolute path, got {:?}",
        single[0]
    );
    let expected_single = std::fs::canonicalize("./src/test_data/pkg_list").unwrap();
    assert_eq!(std::fs::canonicalize(&single[0]).unwrap(), expected_single);

    // Recursive walk: the returned entries must all be absolute and
    // resolve to real directories.
    let recursive = get_pkg_list("./src/test_data/pkg_list/...").unwrap();
    assert_eq!(recursive.len(), 3);
    for entry in &recursive {
        let p = PathBuf::from(entry);
        assert!(p.is_absolute(), "expected absolute path, got {:?}", entry);
        assert!(
            std::fs::canonicalize(&p).unwrap().is_dir(),
            "{:?} is not a directory",
            entry
        );
    }
}

#[test]
fn test_lookup_walkup_kpm_pkg() {
    let root = lookup_walkup_dir("kpm_pkg");
    let a_k = root.join("pkg").join("a.k");
    let tool = toolchain::default();

    assert_eq!(
        lookup_compile_unit_path(a_k.to_str().unwrap()).unwrap(),
        CompileUnitPath::ModFile(root.clone())
    );

    let (files, opts, _) = lookup_compile_workspace(&tool, a_k.to_str().unwrap(), true);
    assert_eq!(files, vec!["main.k".to_string()]);
    assert_eq!(
        opts.map(|o| o.work_dir).unwrap_or_default(),
        root.to_string_lossy().to_string()
    );

    let workspaces =
        lookup_compile_workspace_bounded(&tool, a_k.to_str().unwrap(), true, None).unwrap();
    assert_eq!(workspaces.len(), 1);
    let (kind, (files, _, _)) = workspaces.into_iter().next().unwrap();
    assert_eq!(kind, WorkSpaceKind::ModFile(root.join("kcl.mod")));
    assert_eq!(files, vec!["main.k".to_string()]);
}

#[test]
fn test_lookup_walkup_konfig_like() {
    let root = lookup_walkup_dir("konfig_like");
    let base_a_k = root.join("base").join("pkg1").join("a.k");
    let prod_main_k = root.join("prog").join("prod").join("main.k");
    let tool = toolchain::default();

    // The old format root kcl.mod is not a compile unit root, so the lookup
    // walks past it and falls back to the directory containing the file.
    assert_eq!(
        lookup_compile_unit_path(base_a_k.to_str().unwrap()).unwrap(),
        CompileUnitPath::NotFound
    );

    let (files, _, _) = lookup_compile_workspace(&tool, base_a_k.to_str().unwrap(), true);
    assert_eq!(files, vec![base_a_k.to_string_lossy().to_string()]);

    let workspaces =
        lookup_compile_workspace_bounded(&tool, base_a_k.to_str().unwrap(), true, None).unwrap();
    assert_eq!(workspaces.len(), 1);
    let (kind, _) = workspaces.into_iter().next().unwrap();
    assert_eq!(kind, WorkSpaceKind::NotFound);

    // The kcl.yaml in the same directory as the file is hit before the
    // invalid root kcl.mod.
    assert_eq!(
        lookup_compile_unit_path(prod_main_k.to_str().unwrap()).unwrap(),
        CompileUnitPath::SettingFile(root.join("prog").join("prod"))
    );

    let (files, _, _) = lookup_compile_workspace(&tool, prod_main_k.to_str().unwrap(), true);
    assert_eq!(files, vec!["main.k".to_string()]);

    let workspaces =
        lookup_compile_workspace_bounded(&tool, prod_main_k.to_str().unwrap(), true, None).unwrap();
    assert_eq!(workspaces.len(), 1);
    let (kind, _) = workspaces.into_iter().next().unwrap();
    assert_eq!(
        kind,
        WorkSpaceKind::SettingFile(root.join("prog").join("prod").join("kcl.yaml"))
    );
}

#[test]
fn test_lookup_walkup_plain() {
    let root = lookup_walkup_dir("plain");
    let c_k = root.join("sub").join("c.k");
    let d_k = root.join("sub").join("d.k");
    let tool = toolchain::default();

    assert_eq!(
        lookup_compile_unit_path(c_k.to_str().unwrap()).unwrap(),
        CompileUnitPath::NotFound
    );

    let (mut files, _, _) = lookup_compile_workspace(&tool, c_k.to_str().unwrap(), true);
    files.sort();
    assert_eq!(
        files,
        vec![
            c_k.to_string_lossy().to_string(),
            d_k.to_string_lossy().to_string()
        ]
    );

    let (files, _, _) = lookup_compile_workspace(&tool, c_k.to_str().unwrap(), false);
    assert_eq!(files, vec![c_k.to_string_lossy().to_string()]);
}

#[test]
fn test_lookup_walkup_nested_bound() {
    let root = lookup_walkup_dir("nested_bound");
    let sub = root.join("sub");
    let inner_main_k = sub.join("inner").join("main.k");
    let tool = toolchain::default();

    // Without a bound, the walk-up reaches the root kcl.mod.
    assert_eq!(
        lookup_compile_unit_path(inner_main_k.to_str().unwrap()).unwrap(),
        CompileUnitPath::ModFile(root.clone())
    );
    let (files, _, _) = lookup_compile_workspace(&tool, inner_main_k.to_str().unwrap(), true);
    assert_eq!(files, vec!["main.k".to_string()]);

    // A bound below the kcl.mod truncates the walk-up and falls back to the
    // directory containing the file.
    assert_eq!(
        lookup_compile_unit_path_bounded(inner_main_k.to_str().unwrap(), Some(sub.as_path()))
            .unwrap(),
        CompileUnitPath::NotFound
    );
    let workspaces = lookup_compile_workspace_bounded(
        &tool,
        inner_main_k.to_str().unwrap(),
        true,
        Some(sub.as_path()),
    )
    .unwrap();
    assert_eq!(workspaces.len(), 1);
    let (kind, (files, _, _)) = workspaces.into_iter().next().unwrap();
    assert_eq!(kind, WorkSpaceKind::NotFound);
    assert_eq!(files, vec![inner_main_k.to_string_lossy().to_string()]);

    // A bound at the kcl.mod directory itself still hits the kcl.mod.
    assert_eq!(
        lookup_compile_unit_path_bounded(inner_main_k.to_str().unwrap(), Some(root.as_path()))
            .unwrap(),
        CompileUnitPath::ModFile(root.clone())
    );
    let workspaces = lookup_compile_workspace_bounded(
        &tool,
        inner_main_k.to_str().unwrap(),
        true,
        Some(root.as_path()),
    )
    .unwrap();
    assert_eq!(workspaces.len(), 1);
    let (kind, (files, _, _)) = workspaces.into_iter().next().unwrap();
    assert_eq!(kind, WorkSpaceKind::ModFile(root.join("kcl.mod")));
    assert_eq!(files, vec!["main.k".to_string()]);
}

#[test]
fn test_lookup_walkup_mod_yaml_priority() {
    let root = lookup_walkup_dir("mod_yaml_priority");
    let plain_k = root.join("plain_file.k");
    let tool = toolchain::default();

    // A valid kcl.mod has higher priority than a kcl.yaml in the same
    // directory.
    assert_eq!(
        lookup_compile_unit_path(plain_k.to_str().unwrap()).unwrap(),
        CompileUnitPath::ModFile(root.clone())
    );

    let (files, _, _) = lookup_compile_workspace(&tool, plain_k.to_str().unwrap(), true);
    assert_eq!(files, vec!["mod_main.k".to_string()]);
}

#[test]
fn test_lookup_walkup_kcl_work() {
    let root = lookup_walkup_dir("kcl_work_proj");
    let main_k = root.join("a").join("main.k");
    let sub_b_k = root.join("a").join("sub").join("b.k");
    let tool = toolchain::default();

    assert_eq!(
        lookup_compile_unit_path(sub_b_k.to_str().unwrap()).unwrap(),
        CompileUnitPath::WorkFile(root.join("kcl.work"))
    );

    let workspaces =
        lookup_compile_workspace_bounded(&tool, main_k.to_str().unwrap(), true, None).unwrap();
    assert_eq!(workspaces.len(), 1);
    let (kind, (files, _, _)) = workspaces.into_iter().next().unwrap();
    assert_eq!(kind, WorkSpaceKind::Folder(root.join("a")));
    assert!(files.is_empty());

    // A file directly in a listed workspace falls back to the directory
    // containing the file, as before the walk-up change.
    let (files, _, _) = lookup_compile_workspace(&tool, main_k.to_str().unwrap(), true);
    assert_eq!(files, vec![main_k.to_string_lossy().to_string()]);

    // A file not in any listed workspace falls back to the directory
    // containing the file instead of expanding the work file forever.
    let (files, _, _) = lookup_compile_workspace(&tool, sub_b_k.to_str().unwrap(), true);
    assert_eq!(files, vec![sub_b_k.to_string_lossy().to_string()]);
}
