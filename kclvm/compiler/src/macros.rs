// Copyright 2021 The KCL Authors. All rights reserved.

#[macro_export]
macro_rules! check_backtrack_stop {
    ($ctx: expr) => {
        if let Some(backtrack_meta) = $ctx.backtrack_meta.borrow_mut().as_ref() {
            if backtrack_meta.stop {
                return $ctx.ok_result();
            }
        }
    };
}

#[macro_export]
macro_rules! pkgpath_without_prefix {
    ($pkgpath: expr) => {
        match $pkgpath.strip_prefix('@') {
            Some(v) => v.to_string(),
            None => $pkgpath.to_string(),
        }
    };
}
