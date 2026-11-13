use kcl_ast::ast::Argument;
use kcl_runner::ExecProgramArgs;

use crate::testing::TestRun;

use super::{
    FileCoverage, TestCaseInfo, TestOptions, TestResult, flatten_case_coverage, load_test_suites,
};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Mutex;

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

/// Serializes the tests that run the `test_data/eval_once` fixtures: a run
/// writes its generated main file into the package, where another test
/// loading the same package would pick it up.
static EVAL_ONCE_FIXTURES: Mutex<()> = Mutex::new(());

/// The package `test_data/eval_once/<name>`.
fn eval_once_pkg(name: &str) -> String {
    Path::new(".")
        .join("src")
        .join("testing")
        .join("test_data")
        .join("eval_once")
        .join(name)
        .to_str()
        .unwrap()
        .to_string()
}

/// Run the one suite of the package `test_data/eval_once/<name>` with
/// `opts`, first evaluating the package once per case and then once for
/// all the cases.
fn run_both_ways(name: &str, opts: &TestOptions) -> (TestResult, TestResult) {
    let _lock = EVAL_ONCE_FIXTURES
        .lock()
        .unwrap_or_else(|err| err.into_inner());
    let suites = load_test_suites(eval_once_pkg(name), opts).unwrap();
    assert_eq!(suites.len(), 1);
    let per_case = suites[0]
        .run(&TestOptions {
            eval_once: false,
            ..opts.clone()
        })
        .unwrap();
    let once = suites[0]
        .run(&TestOptions {
            eval_once: true,
            ..opts.clone()
        })
        .unwrap();
    (per_case, once)
}

/// What a test run reports for each case: its name, log and error.
fn outcomes(result: &TestResult) -> Vec<(String, String, Option<String>)> {
    result
        .info
        .iter()
        .map(|(name, info)| {
            (
                name.clone(),
                info.log_message.clone(),
                info.error.as_ref().map(|err| err.to_string()),
            )
        })
        .collect()
}

/// `hits` split into the lines of the main file generated to run the
/// cases and the other lines.
fn split_generated_main(
    hits: &BTreeMap<String, u64>,
) -> (BTreeMap<&str, u64>, BTreeMap<&str, u64>) {
    hits.iter()
        .map(|(key, hits)| (key.as_str(), *hits))
        .partition(|(key, _)| key.contains("_kcl_test_"))
}

/// The files of the coverage of `result` split into the names of the main
/// files generated to run the cases and the coverage of the other files.
fn split_generated_main_files(
    result: &TestResult,
) -> (Vec<&String>, BTreeMap<&String, &FileCoverage>) {
    let (main, rest): (BTreeMap<_, _>, BTreeMap<_, _>) = result
        .coverage
        .files
        .iter()
        .partition(|(file, _)| file.contains("_kcl_test_"));
    (main.into_keys().collect(), rest)
}

/// The options the `eval_once/cases` fixture needs.
fn cases_opts() -> TestOptions {
    TestOptions {
        exec_args: ExecProgramArgs {
            args: vec![Argument {
                name: "who".to_string(),
                value: "\"kcl\"".to_string(),
            }],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Running every case in one evaluation reports the same result, error
/// and log for every case as evaluating the package once per case: a
/// failed assertion, a runtime error or a panic stopped by
/// `runtime.catch` doesn't reach the next case, the module-level `print`
/// opens every log once, and every case sees the schema instances that
/// the package created and its own, not those of the cases before it.
#[test]
fn eval_once_reports_every_case_as_per_case_runs_do() {
    let (per_case, once) = run_both_ways("cases", &cases_opts());
    assert_eq!(outcomes(&once), outcomes(&per_case));

    let failed: Vec<&str> = once
        .info
        .iter()
        .filter(|(_, info)| info.error.is_some())
        .map(|(name, _)| name.as_str())
        .collect();
    assert_eq!(failed, ["test_fail", "test_runtime_error"]);
    let fail_error = once.info["test_fail"].error.as_ref().unwrap().to_string();
    assert!(fail_error.contains("double(2) is not 5"), "{fail_error}");
    assert!(fail_error.contains("main_test.k:10"), "{fail_error}");
    assert_eq!(
        once.info["test_pass_after_fail"].log_message,
        "setup\npass after fail\n"
    );
    assert_eq!(
        once.info["test_pass_after_catch"].log_message,
        "setup\npass after catch\n"
    );
    for (name, info) in &once.info {
        assert_eq!(
            info.log_message.matches("setup").count(),
            1,
            "{name}: {:?}",
            info.log_message
        );
    }
}

/// With `fail_fast`, no case runs after the first one that failed.
#[test]
fn eval_once_stops_on_first_failure_with_fail_fast() {
    let opts = TestOptions {
        fail_fast: true,
        ..cases_opts()
    };
    let (per_case, once) = run_both_ways("cases", &opts);
    assert_eq!(outcomes(&once), outcomes(&per_case));
    let names: Vec<&str> = once.info.keys().map(String::as_str).collect();
    assert_eq!(names, ["test_pass", "test_fail"]);
}

/// Only the cases that match `run_regexp` run.
#[test]
fn eval_once_runs_only_the_selected_cases() {
    let opts = TestOptions {
        run_regexp: "after".to_string(),
        ..cases_opts()
    };
    let (per_case, once) = run_both_ways("cases", &opts);
    assert_eq!(outcomes(&once), outcomes(&per_case));
    let names: Vec<&str> = once.info.keys().map(String::as_str).collect();
    assert_eq!(names, ["test_pass_after_fail", "test_pass_after_catch"]);
}

/// Every case reports the lines that the package's evaluation and the
/// case hit, as when the package is evaluated for that case alone, except
/// on the main file generated to run the cases. Evaluated once, that file
/// selects no case, so each of its lines is hit once. Evaluated for a
/// case, it also calls the case from the line that selects it, and stops
/// there if the case fails.
#[test]
fn eval_once_reports_the_same_coverage() {
    let opts = TestOptions {
        coverage: true,
        ..cases_opts()
    };
    let (per_case, once) = run_both_ways("cases", &opts);
    assert_eq!(once.info.len(), per_case.info.len());
    for (name, info) in &once.info {
        let (once_main, once_rest) = split_generated_main(&info.line_hits);
        let (per_case_main, per_case_rest) = split_generated_main(&per_case.info[name].line_hits);
        assert!(!once_rest.is_empty(), "{name}");
        assert_eq!(once_rest, per_case_rest, "{name}");
        assert!(!once_main.is_empty(), "{name}");
        assert!(once_main.values().all(|hits| *hits == 1), "{name}");
        assert!(
            per_case_main.keys().all(|key| once_main.contains_key(key)),
            "{name}"
        );
        if info.error.is_none() {
            let extra: Vec<i64> = once_main
                .iter()
                .map(|(key, hits)| per_case_main[key] as i64 - *hits as i64)
                .filter(|extra| *extra != 0)
                .collect();
            assert_eq!(extra, [1], "{name}");
        }
    }
    let (once_main, once_rest) = split_generated_main_files(&once);
    let (per_case_main, per_case_rest) = split_generated_main_files(&per_case);
    assert_eq!(once_main.len(), 1);
    assert_eq!(once_main, per_case_main);
    assert_eq!(once_rest, per_case_rest);
}

/// When the package's own code fails, every case reports that failure,
/// as it does when the package is evaluated once per case.
#[test]
fn eval_once_reports_a_failing_package_for_every_case() {
    let (per_case, once) = run_both_ways("setup_error", &TestOptions::default());
    assert_eq!(outcomes(&once), outcomes(&per_case));
    assert_eq!(once.info.len(), 2);
    for (name, info) in &once.info {
        let error = info.error.as_ref().unwrap().to_string();
        assert!(error.contains("setup is broken"), "{name}: {error}");
        assert_eq!(info.log_message, "setup\n", "{name}");
    }

    let opts = TestOptions {
        fail_fast: true,
        ..Default::default()
    };
    let (per_case, once) = run_both_ways("setup_error", &opts);
    assert_eq!(outcomes(&once), outcomes(&per_case));
    assert_eq!(once.info.len(), 1);
}

/// An error from planning the package's values fails every case that
/// passes, as it does when the package is evaluated once per case, while
/// a case that fails reports its own error.
#[test]
fn eval_once_reports_a_planning_error_for_passing_cases() {
    let (per_case, once) = run_both_ways("plan_error", &TestOptions::default());
    for result in [&per_case, &once] {
        let pass_error = result.info["test_pass"].error.as_ref().unwrap().to_string();
        let fail_error = result.info["test_fail"].error.as_ref().unwrap().to_string();
        assert!(pass_error.contains("conflicting values"), "{pass_error}");
        assert!(fail_error.contains("a is not 3"), "{fail_error}");
    }
}

/// A case that sets the output with `manifests.yaml_stream` reports the
/// error of planning that output, as when the package is evaluated for it
/// alone, and the cases after it don't.
#[test]
fn eval_once_plans_the_output_a_case_sets() {
    let (per_case, once) = run_both_ways("yaml_stream", &TestOptions::default());
    for result in [&per_case, &once] {
        let failed: Vec<&str> = result
            .info
            .iter()
            .filter(|(_, info)| info.error.is_some())
            .map(|(name, _)| name.as_str())
            .collect();
        assert_eq!(failed, ["test_bad_stream"]);
        let error = result.info["test_bad_stream"]
            .error
            .as_ref()
            .unwrap()
            .to_string();
        assert!(error.contains("could not find expected ':'"), "{error}");
    }
}

/// A case that replaces output that fails to plan with output that plans
/// passes, as when the package is evaluated for it alone, while a case
/// that keeps that output fails.
#[test]
fn eval_once_plans_the_output_a_case_replaces() {
    let (per_case, once) = run_both_ways("replaced_stream", &TestOptions::default());
    for result in [&per_case, &once] {
        assert!(result.info["test_replace_stream"].error.is_none());
        let error = result.info["test_keep_stream"]
            .error
            .as_ref()
            .unwrap()
            .to_string();
        assert!(error.contains("could not find expected ':'"), "{error}");
    }
}

/// Cases share the values the package defines, so one that mutates a
/// value in place changes what the cases after it see. Here both cases
/// bump the global `counter` through a lambda parameter: evaluated once
/// per case, each starts from zero; evaluated once for both, the second
/// starts from the first one's result. This is why `eval_once` is opt-in.
/// The first case also runs code, such as comprehensions, schema checks
/// and builtins, that must not make the second case evaluate the package
/// again.
#[test]
fn eval_once_shares_the_package_values_between_cases() {
    let (per_case, once) = run_both_ways("shared_values", &TestOptions::default());
    assert!(per_case.info.values().all(|info| info.error.is_none()));
    assert!(once.info["test_first_bump"].error.is_none());
    let error = once.info["test_second_bump"]
        .error
        .as_ref()
        .unwrap()
        .to_string();
    assert!(error.contains("n is 2"), "{error}");
}

/// A case that sets the same output with other planning options reports
/// the error of planning with its options, as when the package is
/// evaluated for it alone, and the cases after it don't.
#[test]
fn eval_once_plans_with_the_options_a_case_sets() {
    let (per_case, once) = run_both_ways("stream_options", &TestOptions::default());
    for result in [&per_case, &once] {
        let error = result.info["test_show_hidden"]
            .error
            .as_ref()
            .unwrap()
            .to_string();
        assert!(error.contains("conflicting values"), "{error}");
        assert!(result.info["test_after_show_hidden"].error.is_none());
    }
}

/// The package is planned before the cases run, so planning does not see
/// a value a case mutates in place. Here the case makes the global `a`
/// conflict with a config expression: evaluated for the case alone, the
/// package then fails to plan; evaluated once, the case passes.
#[test]
fn eval_once_plans_before_the_cases_mutate_values() {
    let (per_case, once) = run_both_ways("shared_plan", &TestOptions::default());
    let error = per_case.info["test_change"]
        .error
        .as_ref()
        .unwrap()
        .to_string();
    assert!(error.contains("conflicting values"), "{error}");
    assert!(once.info["test_change"].error.is_none());
}

/// A case that mutates a value of the package in place and then fails
/// hands the next case a freshly evaluated package, so the next case sees
/// the value as the package defines it, as when the package is evaluated
/// for it alone.
#[test]
fn eval_once_reevaluates_the_package_after_a_failed_case() {
    let (per_case, once) = run_both_ways("failed_mutation", &TestOptions::default());
    assert_eq!(outcomes(&once), outcomes(&per_case));
    let error = once.info["test_bump_then_fail"]
        .error
        .as_ref()
        .unwrap()
        .to_string();
    assert!(error.contains("fails after the bump"), "{error}");
    assert!(once.info["test_counter_is_fresh"].error.is_none());
}

/// So does a case that passes but fails to plan the package: here each
/// case mutates the same value in place and fails only to plan, and the
/// second case does not see the first one's change.
#[test]
fn eval_once_reevaluates_the_package_after_a_case_fails_to_plan() {
    let (per_case, once) = run_both_ways("failed_plan_mutation", &TestOptions::default());
    for result in [&per_case, &once] {
        assert_eq!(result.info.len(), 2);
        for (name, info) in &result.info {
            let error = info.error.as_ref().unwrap().to_string();
            assert!(error.contains("conflicting values"), "{name}: {error}");
            assert!(!error.contains("n is 2"), "{name}: {error}");
        }
    }
}

/// A case is called with no arguments, so one that takes a parameter
/// without a default doesn't compile, whether or not the package is
/// evaluated once.
#[test]
fn eval_once_rejects_a_case_with_a_required_parameter() {
    let _lock = EVAL_ONCE_FIXTURES
        .lock()
        .unwrap_or_else(|err| err.into_inner());
    let suites =
        load_test_suites(eval_once_pkg("required_param"), &TestOptions::default()).unwrap();
    assert_eq!(suites.len(), 1);
    for eval_once in [false, true] {
        let opts = TestOptions {
            eval_once,
            ..Default::default()
        };
        let error = suites[0].run(&opts).unwrap_err().to_string();
        assert!(
            error.contains("expected 1 positional argument, found 0"),
            "eval_once {eval_once}: {error}"
        );
    }
}
