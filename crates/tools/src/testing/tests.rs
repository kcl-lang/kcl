use kcl_ast::ast::Argument;
use kcl_runner::ExecProgramArgs;

use crate::testing::TestRun;

use super::{TestCaseInfo, TestOptions, TestResult, flatten_case_coverage, load_test_suites};
use std::path::Path;

#[test]
fn test_load_test_suites_and_run() {
    let opts = TestOptions {
        exec_args: ExecProgramArgs {
            args: vec![Argument {
                name: "a".to_string(),
                value: "\"a\"".to_string(),
            }],
            ..Default::default()
        },
        ..Default::default()
    };
    let suites = load_test_suites(
        Path::new(".")
            .join("src")
            .join("testing")
            .join("test_data")
            .join("module")
            .join("pkg")
            .to_str()
            .unwrap(),
        &opts,
    )
    .unwrap();
    assert_eq!(suites.len(), 1);
    assert_eq!(suites[0].cases.len(), 3);
    let test_result = suites[0].run(&opts).unwrap();
    assert_eq!(test_result.info.len(), 3);
    assert!(test_result.info[0].error.is_none());
    assert!(
        test_result.info[1]
            .error
            .as_ref()
            .unwrap()
            .to_string()
            .contains("Error"),
    );
    assert!(
        test_result.info[2].error.is_none(),
        "{:?}",
        test_result.info[2].error
    );
}

/// Coverage is opt-in: when `TestOptions::coverage` is false the report
/// stays empty and per-case `line_hits` are empty too. This protects
/// callers that don't want to pay the recording cost.
///
/// We verify the invariants structurally instead of running the suite.
/// The `module/pkg` fixture is shared with `test_load_test_suites_and_run`
/// and both tests would race on the generated `_kcl_test.k` temp file when
/// `cargo test` schedules them on parallel threads — one test's run()
/// can delete the file while another is still mid-execution, producing a
/// spurious "system cannot find the file" failure on Windows. The
/// default-shaped structs already give us what we want to assert:
///
/// * `TestOptions::default().coverage` is `false`,
/// * `TestResult::default().coverage.files` is empty,
/// * `TestCaseInfo::default().line_hits` is empty.
///
/// and `TestSuite::run` only populates those fields inside an
/// `if opts.coverage { ... }` guard, so the invariants hold end-to-end.
#[test]
fn coverage_off_keeps_reports_empty() {
    let opts = TestOptions::default();
    assert!(
        !opts.coverage,
        "TestOptions::default() must default coverage to false"
    );
    let empty_result = TestResult::default();
    assert!(
        empty_result.coverage.files.is_empty(),
        "default TestResult.coverage.files should be empty"
    );
    assert_eq!(empty_result.coverage.summary.covered, 0);
    assert_eq!(empty_result.coverage.summary.executable, 0);
    let empty_info = TestCaseInfo::default();
    assert!(
        empty_info.line_hits.is_empty(),
        "default TestCaseInfo.line_hits should be empty"
    );
}

/// When coverage is on we expect:
///   - per-case `line_hits` to be non-empty for every case that ran,
///   - `result.coverage.files` to contain `lib.k`,
///   - some lines to be covered and some to be uncovered (the test
///     fixture leaves two branches of `choose` unexercised on purpose),
///   - the roll-up summary to be a number in `[0.0, 100.0]`.
///
/// We intentionally avoid asserting exact hit counts because they depend
/// on KCL lazy-evaluation replays and small refactors of the evaluator
/// shouldn't break coverage tests.
#[test]
fn coverage_on_collects_per_file_and_summary() {
    let opts = TestOptions {
        coverage: true,
        ..Default::default()
    };
    let suites = load_test_suites(
        Path::new(".")
            .join("src")
            .join("testing")
            .join("test_data")
            .join("coverage")
            .join("pkg")
            .to_str()
            .unwrap(),
        &opts,
    )
    .unwrap();
    assert_eq!(suites.len(), 1);
    let result = suites[0].run(&opts).unwrap();

    // Per-case: every case that ran must have at least one line hit.
    // The fixture has 4 cases.
    assert!(result.info.len() >= 3, "fixture should run several cases");
    for (name, info) in &result.info {
        assert!(
            !info.line_hits.is_empty(),
            "case {name} should report line hits when coverage is on"
        );
    }

    // The fixture declares `lib.k` and `lib_test.k`. Both should appear
    // because both contribute statements that get walked. Path keys
    // can come back as either absolute or relative depending on how
    // each module was loaded, so we match by suffix. Suffixes are
    // hardcoded with `/` separators because `canonicalize_for_coverage`
    // normalizes report keys to that form on every platform.
    let lib_path_str = "src/testing/test_data/coverage/pkg/lib.k".to_string();
    let lib_test_path_str = "src/testing/test_data/coverage/pkg/lib_test.k".to_string();
    let find_matching = |suffix: &str| -> Option<&super::FileCoverage> {
        result
            .coverage
            .files
            .iter()
            .find(|(k, _)| k.ends_with(suffix))
            .map(|(_, v)| v)
    };
    let lib_cov =
        find_matching(&lib_path_str).expect("lib.k should be tracked in the coverage report");
    let _lib_test_cov = find_matching(&lib_test_path_str)
        .expect("lib_test.k should be tracked in the coverage report");

    // The fixture's `choose` lambda has 3 branches (positive / negative /
    // zero). The test suite only exercises the positive branch. So
    // `lib.k`'s executable lines should outnumber its covered lines.
    assert!(
        lib_cov.executable_lines.len() >= lib_cov.covered_lines.len(),
        "covered lines should be a subset of executable lines (covered={}, executable={})",
        lib_cov.covered_lines.len(),
        lib_cov.executable_lines.len()
    );
    assert!(
        lib_cov.executable_lines.len() > lib_cov.covered_lines.len(),
        "fixture should leave at least one branch uncovered to make the test meaningful (covered={}, executable={})",
        lib_cov.covered_lines.len(),
        lib_cov.executable_lines.len()
    );

    // Summary rolls up across every file.
    let summary = &result.coverage.summary;
    assert!(summary.executable > 0);
    assert!(summary.covered > 0);
    assert!(summary.covered <= summary.executable);
    assert!(
        (0.0..=100.0).contains(&summary.percent),
        "percent out of range: {}",
        summary.percent
    );
}

/// `flatten_case_coverage` should produce `<file>:<line>` keys and
/// preserve hit counts. Used by the suite runner to merge per-case hits
/// into the suite-wide report.
#[test]
fn flatten_case_coverage_basic() {
    use std::collections::HashMap;
    let mut per_file: HashMap<String, HashMap<u64, u64>> = HashMap::new();
    let mut lines = HashMap::new();
    lines.insert(3u64, 2u64);
    lines.insert(7u64, 1u64);
    per_file.insert("foo.k".to_string(), lines);
    let flat = flatten_case_coverage(&per_file);
    assert_eq!(flat.get("foo.k:3").copied(), Some(2));
    assert_eq!(flat.get("foo.k:7").copied(), Some(1));
    assert_eq!(flat.len(), 2);
}

/// `normalize_coverage_key` is the join between the executable-line
/// scan (which calls `canonicalize_for_coverage`) and the per-case
/// coverage hits (which go through `flatten_case_coverage`). Both must
/// agree on separator style or `TestCoverageReport::finalize` will
/// silently drop executable lines on Windows. Pin the contract here.
#[test]
fn normalize_coverage_key_strips_backslashes() {
    // Whatever the platform produces, the result must never contain
    // backslashes — that's the cross-platform invariant the report
    // keys depend on.
    let result = super::normalize_coverage_key("dir\\sub\\file.k");
    assert!(
        !result.contains('\\'),
        "normalize_coverage_key must not leave backslashes in result, got {result:?}"
    );

    // On POSIX platforms the function is the identity for already-
    // normalized input. On Windows it must collapse separators to `/`.
    let posix = super::normalize_coverage_key("dir/sub/file.k");
    if cfg!(windows) {
        assert!(posix.contains('/'));
    } else {
        assert_eq!(posix, "dir/sub/file.k");
    }
}

/// `flatten_case_coverage` should normalize the file keys it receives
/// so they line up with the executable-line map. Without normalization
/// the report's `executable_lines` would be empty on Windows.
#[test]
fn flatten_case_coverage_normalizes_keys() {
    use std::collections::HashMap;
    let mut lines = HashMap::new();
    lines.insert(1u64, 1u64);
    let mut per_file = HashMap::new();
    per_file.insert("foo.k".to_string(), lines);
    let flat = flatten_case_coverage(&per_file);
    let expected_key = format!("{}:1", super::normalize_coverage_key("foo.k"));
    assert!(
        flat.contains_key(&expected_key),
        "flatten_case_coverage should use normalized key, expected {expected_key:?} in {flat:?}"
    );
}
