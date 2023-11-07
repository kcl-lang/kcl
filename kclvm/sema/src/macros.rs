//! Copyright The KCL Authors. All rights reserved.

#[macro_export]
macro_rules! pkgpath_without_prefix {
    ($pkgpath: expr) => {
        match $pkgpath.strip_prefix('@') {
            Some(v) => v.to_string(),
            None => $pkgpath.to_string(),
        }
    };
}
