// Copyright 2021 The KCL Authors. All rights reserved.

pub const VERSION: &str = "0.5.0-alpha.4";
pub const CHECK_SUM: &str = "51289ff122a3ea8eea7f8149b8956cd1";

/// Get kCL full version string with the format `{version}-{check_sum}`.
#[inline]
pub fn get_version_string() -> String {
    format!("{}-{}", VERSION, CHECK_SUM)
}

/// Get version info including version string, platform.
#[inline]
pub fn get_version_info() -> String {
    format!(
        "Version: {}\r\nPlatform: {}\r\nGitCommit: {}",
        get_version_string(),
        env!("VERGEN_RUSTC_HOST_TRIPLE"),
        env!("VERGEN_GIT_SHA")
    )
}
