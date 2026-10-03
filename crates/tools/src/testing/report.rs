//! Human-readable rendering of [`TestResult`] for the testing tool.
//!
//! [`PrettyReporter`] renders a [`TestResult`] into the same text format as
//! the kcl-go `PrettyReporter`, so every KCL language binding can print
//! byte-identical test reports.
use crate::testing::TestResult;

/// Width of the separator line emitted between the case list and the
/// summary counts.
const SEPARATOR_WIDTH: usize = 80;

/// Renders [`TestResult`] values in a simple human-readable format.
///
/// The output is byte-identical to the kcl-go `PrettyReporter` and is
/// deterministic for a given result: no timestamps, no host-specific paths,
/// no locale-sensitive formatting.
///
/// Format (every line, including the last one, ends with `\n`):
///
/// ```text
/// {name}: {STATUS} ({duration_ms}ms)
/// ```
///
/// one line per case in result order, where `STATUS` is `PASS` or `FAIL`
/// (no native case can currently be skipped; the `SKIPPED` plumbing is kept
/// for parity with the Go reporter) and `duration_ms` is the case duration
/// in microseconds **truncated** to whole milliseconds (integer division,
/// e.g. 1500µs renders as `1ms`). When a case has a non-empty log message,
/// the log is appended on the following line; otherwise a failed case
/// appends its error string as-is (the error already carries its own
/// prefix, e.g. `Error: ...`).
///
/// After all cases, a separator line of exactly 80 `-` characters is
/// emitted, followed by the non-zero counts in this order:
///
/// ```text
/// PASS: {p}/{total}
/// FAIL: {f}/{total}
/// SKIPPED: {s}/{total}
/// ```
///
/// where `total` is the number of cases.
///
/// When the result is empty — no cases and no coverage — the report is
/// exactly `no test files\n` (matching the message the Go CLI prints for an
/// empty run), so bindings inherit the behavior.
///
/// When coverage is populated ([`TestResult.coverage.files`] is non-empty),
/// the summary lines are followed by one roll-up line built from the
/// pre-computed [`CoverageSummary`](crate::testing::CoverageSummary) fields:
///
/// ```text
/// Coverage: {percent:.1}% ({covered}/{executable} lines)
/// ```
///
/// and then one line per file, sorted by filename, indented two spaces:
///
/// ```text
///   {filename}: {covered_lines}/{executable_lines} ({percent:.1}%)
/// ```
///
/// where the per-file percent is `100.0 * covered / executable`, or `0.0%`
/// when the file has no executable lines.
pub struct PrettyReporter;

impl PrettyReporter {
    /// Render `result` as a human-readable report. See the
    /// [struct-level documentation](PrettyReporter) for the exact format.
    pub fn render(result: &TestResult) -> String {
        // Mirror the Go CLI: an empty run (no cases and no coverage)
        // renders as "no test files".
        if result.info.is_empty() && result.coverage.files.is_empty() {
            return "no test files\n".to_string();
        }
        let mut out = String::new();
        let (mut pass, mut fail) = (0usize, 0usize);
        // No native case can currently be skipped, so `skip` stays 0 and the
        // SKIPPED summary line never prints; the counter and the emission
        // below mirror the Go reporter so the format stays stable if
        // skipping is added later.
        let skip = 0usize;
        for (name, info) in &result.info {
            // A case fails when its error string is non-empty, matching the
            // Go predicate `ErrMessage != ""`. The error text is rendered
            // as-is below; it carries its own prefix.
            let err = info
                .error
                .as_ref()
                .map(|e| e.to_string())
                .unwrap_or_default();
            let status = if err.is_empty() {
                pass += 1;
                "PASS"
            } else {
                fail += 1;
                "FAIL"
            };
            // The duration is truncated, not rounded: Go's reporter divides
            // the microsecond count by 1000 with integer division.
            out.push_str(&format!(
                "{name}: {status} ({}ms)\n",
                info.duration.as_micros() / 1000
            ));
            if !info.log_message.is_empty() {
                out.push_str(&info.log_message);
                out.push('\n');
            } else if !err.is_empty() {
                out.push_str(&err);
                out.push('\n');
            }
        }
        out.push_str(&"-".repeat(SEPARATOR_WIDTH));
        out.push('\n');
        let total = pass + fail + skip;
        if pass != 0 {
            out.push_str(&format!("PASS: {pass}/{total}\n"));
        }
        if fail != 0 {
            out.push_str(&format!("FAIL: {fail}/{total}\n"));
        }
        if skip != 0 {
            out.push_str(&format!("SKIPPED: {skip}/{total}\n"));
        }
        if !result.coverage.files.is_empty() {
            let summary = &result.coverage.summary;
            out.push_str(&format!(
                "Coverage: {:.1}% ({}/{}) lines\n",
                summary.percent, summary.covered, summary.executable
            ));
            for (filename, file_cov) in &result.coverage.files {
                let percent = if file_cov.executable_lines.is_empty() {
                    0.0
                } else {
                    100.0 * file_cov.covered_lines.len() as f64
                        / file_cov.executable_lines.len() as f64
                };
                out.push_str(&format!(
                    "  {filename}: {}/{} ({percent:.1}%)\n",
                    file_cov.covered_lines.len(),
                    file_cov.executable_lines.len()
                ));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{CoverageSummary, FileCoverage, TestCaseInfo, TestCoverageReport};
    use kcl_primitives::IndexMap;
    use std::time::Duration;

    /// Build a case with the given name, error text and duration. An empty
    /// `err` means the case passed. Durations are fixed microseconds so the
    /// rendered milliseconds are deterministic.
    fn case(name: &str, err: &str, micros: u64) -> (String, TestCaseInfo) {
        (
            name.to_string(),
            TestCaseInfo {
                error: if err.is_empty() {
                    None
                } else {
                    Some(anyhow::anyhow!(err.to_string()))
                },
                duration: Duration::from_micros(micros),
                ..Default::default()
            },
        )
    }

    /// The kcl-go `PrettyReporter` golden, with a duration that exercises
    /// the millisecond truncation (1500µs renders as `1ms`, not `2ms`).
    #[test]
    fn renders_go_golden_format() {
        let mut info = IndexMap::default();
        let (k, v) = case("test_not_log_message", "", 1500);
        info.insert(k, v);
        let (k, mut v) = case("test_foo", "", 1500);
        v.log_message = "log message".to_string();
        info.insert(k, v);
        let (k, v) = case("test_bar", "Error: assert failed", 2500);
        info.insert(k, v);
        let result = TestResult {
            info,
            ..Default::default()
        };
        let rendered = PrettyReporter::render(&result);
        // Every line, including the last one, ends with `\n` (insta strips
        // the trailing newline when storing, so pin it explicitly).
        assert!(rendered.ends_with("FAIL: 1/3\n"), "got {rendered:?}");
        insta::assert_snapshot!(rendered);
    }

    /// An empty result (no cases and no coverage) renders exactly the
    /// message the Go CLI prints for an empty run.
    #[test]
    fn renders_no_test_files_for_empty_result() {
        assert_eq!(
            PrettyReporter::render(&TestResult::default()),
            "no test files\n"
        );
    }

    /// Coverage rendering: one roll-up line from the pre-computed summary,
    /// then per-file lines sorted by filename, indented two spaces, with
    /// `0.0%` for files without executable lines.
    #[test]
    fn renders_coverage_report() {
        let mut info = IndexMap::default();
        let (k, mut v) = case("test_alpha", "", 1500);
        v.log_message = "hello from alpha".to_string();
        info.insert(k, v);
        let (k, v) = case("test_beta", "Error: boom", 3100);
        info.insert(k, v);
        let mut files = std::collections::BTreeMap::new();
        files.insert(
            "a.k".to_string(),
            FileCoverage {
                covered_lines: vec![1, 2],
                executable_lines: vec![1, 2, 3],
                ..Default::default()
            },
        );
        files.insert(
            "b.k".to_string(),
            FileCoverage {
                executable_lines: vec![1],
                ..Default::default()
            },
        );
        files.insert("empty.k".to_string(), FileCoverage::default());
        let result = TestResult {
            info,
            coverage: TestCoverageReport {
                files,
                summary: CoverageSummary {
                    covered: 2,
                    executable: 4,
                    percent: 50.0,
                },
            },
        };
        let rendered = PrettyReporter::render(&result);
        assert!(
            rendered.ends_with("  empty.k: 0/0 (0.0%)\n"),
            "got {rendered:?}"
        );
        insta::assert_snapshot!(rendered);
    }
}
