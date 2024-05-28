use std::path::{Path, PathBuf};
use std::process::Command;

use crate::client::fs::directory_is_not_empty;
use anyhow::{bail, Result};

pub(crate) fn clone_git_repo_to(
    url: &str,
    branch: &Option<String>,
    tag: &Option<String>,
    commit: &Option<String>,
    path: &Path,
) -> Result<PathBuf> {
    if directory_is_not_empty(path) {
        return Ok(path.to_path_buf());
    }
    let mut git_clone_cmd = Command::new("git");
    git_clone_cmd.args(["clone", url]);
    if let Some(branch_name) = branch {
        git_clone_cmd.args(["--branch", branch_name]);
    }
    git_clone_cmd.arg(path);

    let output = git_clone_cmd.output()?;
    if !output.status.success() {
        bail!("Failed to clone Git repository {}", url);
    }
    if let Some(tag_name) = tag {
        let output = Command::new("git")
            .args(["checkout", tag_name])
            .current_dir(path)
            .output()?;
        if !output.status.success() {
            bail!("Failed to checkout Git tag");
        }
    } else if let Some(commit_hash) = commit {
        let output = Command::new("git")
            .args(["checkout", commit_hash])
            .current_dir(path)
            .output()?;
        if !output.status.success() {
            bail!("Failed to checkout Git commit");
        }
    }

    Ok(path.to_path_buf())
}
