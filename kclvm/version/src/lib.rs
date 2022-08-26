// Copyright 2021 The KCL Authors. All rights reserved.

pub const VERSION: &str = "0.4.3";
pub const CHECK_SUM: &str = "e07ed7af0d9bd1e86a3131714e4bd20c";

pub fn get_full_version() -> String {
    format!("{}-{}", VERSION, CHECK_SUM)
}
