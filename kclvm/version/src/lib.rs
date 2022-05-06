// Copyright 2021 The KCL Authors. All rights reserved.

pub const VERSION: &str = "0.4.1";
pub const CHECK_SUM: &str = "cc68dfc367e70d516649638f217a53a3";

pub fn get_full_version() -> String {
    format!("{}-{}", VERSION, CHECK_SUM)
}
