// Copyright 2021 The KCL Authors. All rights reserved.

pub const VERSION: &str = "0.4.4";
pub const CHECK_SUM: &str = "c5339e572207211e46477825e8aca903";

pub fn get_full_version() -> String {
    format!("{}-{}", VERSION, CHECK_SUM)
}
