//! Copyright The KCL Authors. All rights reserved.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CHECK_SUM: &str = "c020ab3eb4b9179219d6837a57f5d323";
pub const GIT_SHA: &str = env!("VERGEN_GIT_SHA");
pub const HOST_TRIPLE: &str = env!("VERGEN_RUSTC_HOST_TRIPLE");

/// Get KCL full version string with the format `{version}-{check_sum}`.
#[inline]
pub fn get_version_string() -> String {
    format!("{}-{}", VERSION, CHECK_SUM)
}

/// Get KCL build git sha.
#[inline]
pub fn get_git_sha() -> &'static str {
    option_env!("KCL_BUILD_GIT_SHA").unwrap_or_else(|| GIT_SHA)
}

/// Get version info including version string, platform.
#[inline]
pub fn get_version_info() -> String {
    format!(
        "Version: {}\r\nPlatform: {}\r\nGitCommit: {}",
        get_version_string(),
        HOST_TRIPLE,
        get_git_sha(),
    )
}
