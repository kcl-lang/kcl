use anyhow::bail;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::client::fs::directory_is_not_empty;
use anyhow::Result;
use git2::build::RepoBuilder;
use git2::{Commit, Repository};

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
    let mut builder = RepoBuilder::new();
    if let Some(branch) = branch {
        builder.branch(branch);
    }
    let repo = builder.clone(url, Path::new(&path))?;
    if let Some(tag) = tag {
        let (object, _) = repo.revparse_ext(tag)?;
        repo.checkout_tree(&object, None)?;
        if let Ok(tag) = repo.find_tag(object.id()) {
            let target = tag.target_id();
            repo.set_head_detached(target)?;
        } else {
            repo.set_head_detached(object.id())?;
        }
    } else if let Some(commit) = commit {
        let commit = find_commit_by_prefix(&repo, commit)?;
        repo.checkout_tree(commit.as_object(), None)?;
        repo.set_head_detached(commit.as_object().id())?;
    }
    Ok(path.to_path_buf())
}

fn find_commit_by_prefix<'a>(repo: &'a Repository, prefix: &'a str) -> Result<Commit<'a>> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    for oid in revwalk {
        let oid = oid?;
        if oid.to_string().starts_with(prefix) {
            return Ok(repo.find_commit(oid)?);
        }
    }
    Err(anyhow::anyhow!(
        "No matching commit found for the prefix {prefix}"
    ))
}

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
