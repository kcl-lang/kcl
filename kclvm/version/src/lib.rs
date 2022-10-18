// Copyright 2021 The KCL Authors. All rights reserved.

pub const VERSION: &str = "0.4.3";
pub const CHECK_SUM: &str = "c5bd1f3a5d6db8c676bafddb6e643660";

pub fn get_full_version() -> String {
    format!("{}-{}", VERSION, CHECK_SUM)
}
