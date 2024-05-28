use anyhow::bail;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::client::fs::directory_is_not_empty;
use anyhow::Result;
use kclvm_utils::path::PathPrefix;

pub(crate) fn cmd_clone_git_repo_to(
    url: &str,
    branch: &Option<String>,
    tag: &Option<String>,
    commit: &Option<String>,
    path: &Path,
) -> Result<PathBuf> {
    if directory_is_not_empty(path) {
        return Ok(path.to_path_buf());
    }
    let path = path.adjust_canonicalization();
    let mut git_clone_cmd = Command::new("git");
    git_clone_cmd.args(["clone", url]);
    if let Some(branch_name) = branch {
        git_clone_cmd.args(["--branch", branch_name]);
    }
    git_clone_cmd.arg(&path);

    let output = git_clone_cmd.output()?;
    if !output.status.success() {
        bail!(
            "Failed to clone Git repository {}: stdout: {} stderr: {}",
            url,
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap()
        );
    }
    if let Some(tag_name) = tag {
        let output = Command::new("git")
            .args(["checkout", tag_name])
            .current_dir(&path)
            .output()?;
        if !output.status.success() {
            bail!(
                "Failed to checkout Git tag {}: stdout: {} stderr: {}",
                tag_name,
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            );
        }
    } else if let Some(commit_hash) = commit {
        let output = Command::new("git")
            .args(["checkout", commit_hash])
            .current_dir(&path)
            .output()?;
        if !output.status.success() {
            bail!(
                "Failed to checkout Git commit {}: stdout: {} stderr: {}",
                commit_hash,
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            )
        }
    }

    Ok(path.into())
}
