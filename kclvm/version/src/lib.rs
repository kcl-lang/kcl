// Copyright 2021 The KCL Authors. All rights reserved.

pub const VERSION: &str = "0.4.4";
pub const CHECK_SUM: &str = "9e3303edaba484df6004620bf7b28b98";

pub fn get_full_version() -> String {
    format!("{}-{}", VERSION, CHECK_SUM)
}
