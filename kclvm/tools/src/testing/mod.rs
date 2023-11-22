//! [kclvm_tools::testing] module mainly contains some functions of language testing tool.
//!
//! The basic principle of the testing tool is to search for test files in the KCL package
//! that have the suffix "_test.k" and do not start with "_". These test files will be regard
//! as test suites. Within these files, any lambda literals starting with "test_" will be
//! considered as test cases, but these lambda functions should not have any parameters.
//! To perform the testing, the tool compiles the test suite file and its dependencies into an
//! [kclvm_runner::Artifact], which is regard as a new compilation entry point. Then,
//! it executes each test case separately and collects information about the test cases,
//! such as the execution time and whether the test passes or fails.
pub use crate::testing::suite::{load_test_suites, TestSuite};
use anyhow::{Error, Result};
use indexmap::IndexMap;
use kclvm_runner::ExecProgramArgs;
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
}
