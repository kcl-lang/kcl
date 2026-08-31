//! Line-level coverage tracking for the KCL evaluator.
//!
//! The evaluator is normally a pure walker that evaluates AST statements to
//! produce values. When the caller wants line coverage (for example the
//! `kcl test` tool with `--coverage`), the evaluator can be wired up with a
//! shared [`CoverageState`] via [`Evaluator::set_coverage_state`]. Every time
//! the eager walker enters a top-level statement, the source file path and
//! 1-based line number are recorded. Calls originating from inside a
//! backtracking setter replay are ignored — those are value-only replays and
//! would double-count the same source line.
//!
//! Recording is opt-in and disabled by default so existing call sites pay
//! zero overhead.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Records `(filename, line) -> hit count` for every KCL statement that the
/// eager evaluator pass enters. Cloning is cheap: the inner map is shared
/// across every [`Evaluator`](crate::Evaluator) that wants to feed into the
/// same coverage run.
#[derive(Debug, Default, Clone)]
pub struct CoverageState {
    inner: Rc<RefCell<HashMap<(String, u64), u64>>>,
}

impl CoverageState {
    /// Construct an empty coverage state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one hit for `filename:line`. No-op when either field is empty
    /// — generated source files (such as the `kcl test` dispatch shim) often
    /// have a synthetic filename that we don't want in the report.
    pub fn record_hit(&self, filename: &str, line: u64) {
        if filename.is_empty() || line == 0 {
            return;
        }
        let mut map = self.inner.borrow_mut();
        *map.entry((filename.to_string(), line)).or_insert(0) += 1;
    }

    /// Drain the recorded hits as a `Vec<(filename, line, hits)>` and leave
    /// the state empty. Used by the runner to extract data after each
    /// `exec_program` call without copying the whole map twice.
    pub fn drain(&self) -> Vec<(String, u64, u64)> {
        let mut map = self.inner.borrow_mut();
        let drained: Vec<_> = map.drain().map(|((f, l), hits)| (f, l, hits)).collect();
        drained
    }

    /// Snapshot the current hits without consuming them. Useful when the
    /// caller wants a per-case view alongside an aggregated report.
    pub fn snapshot(&self) -> Vec<(String, u64, u64)> {
        let map = self.inner.borrow();
        let mut out: Vec<_> = map
            .iter()
            .map(|((f, l), hits)| (f.clone(), *l, *hits))
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        out
    }

    /// True when no hits have been recorded. Lets callers short-circuit the
    /// "build a coverage report" path when coverage mode was requested but
    /// nothing actually executed.
    pub fn is_empty(&self) -> bool {
        self.inner.borrow().is_empty()
    }
}
