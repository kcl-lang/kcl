//! [kcl_tools::testing] module mainly contains some functions of language testing tool.
//!
//! The basic principle of the testing tool is to search for test files in the KCL package
//! that have the suffix "_test.k" and do not start with "_". These test files will be regard
//! as test suites. Within these files, any lambda literals starting with "test_" will be
//! considered as test cases, but these lambda functions should not have any parameters.
//! To perform the testing, the tool compiles and executes the test suite file together with
//! its dependencies as a single program entry point. Then, it executes each test case
//! separately and collects information about the test cases, such as the execution time and
//! whether the test passes or fails.
//!
//! When [`TestOptions::coverage`] is enabled, the test tool additionally records
//! line-level coverage for every KCL statement that executes during the run. The
//! aggregated [`TestCoverageReport`] is returned alongside the usual
//! [`TestResult`] so callers can render it in their preferred format.
pub use crate::testing::suite::{TestSuite, load_test_suites};
use anyhow::{Error, Result};
use kcl_primitives::IndexMap;
use kcl_runner::ExecProgramArgs;
use std::collections::BTreeMap;
use std::time::Duration;

mod suite;

#[cfg(test)]
mod tests;

/// Trait for running tests.
pub trait TestRun {
    type Options;
    type Result;

    /// Run the test with the given options and return the result.
    fn run(&self, opts: &Self::Options) -> Result<Self::Result>;
}

/// Represents the result of a test.
#[derive(Debug, Default)]
pub struct TestResult {
    /// This field stores test case information in an [IndexMap], where the key is a [String] and the value is a [TestCaseInfo] struct.
    pub info: IndexMap<String, TestCaseInfo>,
    /// Aggregated line-level coverage across all test cases, populated only
    /// when [`TestOptions::coverage`] was true.
    pub coverage: TestCoverageReport,
}

/// Represents information about a test case.
#[derive(Debug, Default)]
pub struct TestCaseInfo {
    /// This field stores the log message of the test.
    pub log_message: String,
    /// This field stores the error associated with the test case, if any.
    pub error: Option<Error>,
    /// This field stores the duration of the test case.
    pub duration: Duration,
    /// Per-case line hits. Keys use the `<file>:<line>` convention; values
    /// are the number of times that line was entered while running this
    /// case. Empty unless coverage was requested.
    pub line_hits: BTreeMap<String, u64>,
}

/// Represents options for running tests.
#[derive(Debug, Default, Clone)]
pub struct TestOptions {
    /// This field stores the execution program arguments.
    pub exec_args: ExecProgramArgs,
    /// This field stores a regular expression for filtering tests to run.
    pub run_regexp: String,
    /// This field determines whether the test run should stop on the first failure.
    pub fail_fast: bool,
    /// When true, every test case runs with coverage recording turned on
    /// and the aggregated data is exposed via [`TestResult::coverage`].
    /// Defaults to false — coverage recording has a small cost so callers
    /// must opt in.
    pub coverage: bool,
}

/// Per-file coverage entry. `covered_lines` and `executable_lines` are kept
/// sorted and deduplicated for deterministic output.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FileCoverage {
    /// Lines that were entered at least once across the entire run.
    pub covered_lines: Vec<u64>,
    /// Lines in the source file that contain an executable statement. This
    /// is computed from the AST so comments and blank lines are excluded.
    pub executable_lines: Vec<u64>,
    /// Per-line hit count: `line -> count`.
    pub line_hits: BTreeMap<u64, u64>,
}

/// Aggregated coverage across an entire test run.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TestCoverageReport {
    /// Per-file coverage keyed by source file path.
    pub files: BTreeMap<String, FileCoverage>,
    /// Roll-up totals across every file in [`TestCoverageReport::files`].
    pub summary: CoverageSummary,
}

/// Roll-up coverage metrics.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct CoverageSummary {
    /// Number of executable lines that were hit by at least one test.
    pub covered: u64,
    /// Total number of executable lines discovered.
    pub executable: u64,
    /// Coverage percentage in `[0.0, 100.0]`. `0.0` when no executable
    /// lines were discovered (avoids divide-by-zero in display code).
    pub percent: f64,
}

impl TestCoverageReport {
    /// Add the hits recorded by a single test case run. Counts for the
    /// same `(filename, line)` pair are summed so multiple cases that
    /// touch the same line produce a stable total.
    pub fn merge(&mut self, case_hits: &BTreeMap<String, u64>) {
        for (key, hits) in case_hits {
            // Keys are `<file>:<line>` so we split on the last colon to
            // tolerate paths that themselves contain colons (Windows
            // drive letters). Use `rfind` to anchor at the rightmost.
            let (filename, line_str) = match key.rsplit_once(':') {
                Some(parts) => parts,
                None => continue,
            };
            let line = match line_str.parse::<u64>() {
                Ok(n) => n,
                Err(_) => continue,
            };
            let entry = self.files.entry(filename.to_string()).or_default();
            *entry.line_hits.entry(line).or_insert(0) += hits;
        }
    }

    /// After [`merge`](Self::merge) has collected all per-case hits, build
    /// the derived `covered_lines`, `executable_lines`, and `summary`
    /// fields. Call once after the last merge.
    ///
    /// Files present in `executable_lines_by_file` but not yet in
    /// `self.files` (e.g. modules that never executed) are added with
    /// empty `line_hits` so callers see a per-file entry for everything
    /// the package declared.
    pub fn finalize(&mut self, executable_lines_by_file: &BTreeMap<String, Vec<u64>>) {
        // Back-fill missing entries so unhit files show up in the report.
        for (filename, lines) in executable_lines_by_file {
            let entry = self
                .files
                .entry(filename.clone())
                .or_insert_with(|| FileCoverage {
                    executable_lines: lines.clone(),
                    ..Default::default()
                });
            if entry.executable_lines.is_empty() && !lines.is_empty() {
                entry.executable_lines = lines.clone();
            }
        }
        for (filename, file_cov) in self.files.iter_mut() {
            // Prefer the executable map's view; fall back to any
            // previously-stored value (kept around so callers can decide
            // to start an "executable" list by other means).
            let executable = executable_lines_by_file
                .get(filename)
                .cloned()
                .unwrap_or_else(|| file_cov.executable_lines.clone());
            let mut executable_set: std::collections::BTreeSet<u64> =
                executable.iter().copied().collect();
            // Any line that was actually hit but isn't in the executable set
            // probably came from a synthetic location (e.g. injected
            // assertions). Don't drop the hit, but mark it as covered so the
            // user sees something was executed.
            for line in file_cov.line_hits.keys() {
                executable_set.insert(*line);
            }
            let mut covered: Vec<u64> = file_cov
                .line_hits
                .keys()
                .copied()
                .filter(|l| executable_set.contains(l))
                .collect();
            covered.sort_unstable();
            covered.dedup();
            let mut executable_lines: Vec<u64> = executable_set.into_iter().collect();
            executable_lines.sort_unstable();
            executable_lines.dedup();
            file_cov.covered_lines = covered;
            file_cov.executable_lines = executable_lines;
        }
        let mut covered = 0u64;
        let mut executable = 0u64;
        for file_cov in self.files.values() {
            covered += file_cov.covered_lines.len() as u64;
            executable += file_cov.executable_lines.len() as u64;
        }
        let percent = if executable == 0 {
            0.0
        } else {
            (covered as f64) * 100.0 / (executable as f64)
        };
        self.summary = CoverageSummary {
            covered,
            executable,
            percent,
        };
    }
}

/// Convert a single test case's coverage map (`filename -> line -> hits`)
/// into the `<file>:<line>` key convention used by
/// [`TestCoverageReport::merge`]. The conversion is intentionally
/// conservative: empty filenames are skipped, lines must be non-zero.
pub fn flatten_case_coverage(
    per_file: &std::collections::HashMap<String, std::collections::HashMap<u64, u64>>,
) -> BTreeMap<String, u64> {
    let mut out = BTreeMap::new();
    for (filename, lines) in per_file {
        if filename.is_empty() {
            continue;
        }
        for (line, hits) in lines {
            if *line == 0 {
                continue;
            }
            out.insert(format!("{}:{}", filename, line), *hits);
        }
    }
    out
}
