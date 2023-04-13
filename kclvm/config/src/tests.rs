use std::{env, path::PathBuf};

use crate::modfile::{get_vendor_home, KCLVM_VENDOR_HOME};

#[test]
fn test_vendor_home() {
    env::set_var(KCLVM_VENDOR_HOME, "test_vendor_home");
    assert_eq!(get_vendor_home(), "test_vendor_home");
    env::remove_var(KCLVM_VENDOR_HOME);

    let root_dir = env::var("HOME").unwrap();
    let kpm_home = PathBuf::from(root_dir).join(".kpm");
    assert_eq!(get_vendor_home(), kpm_home.display().to_string())
}
