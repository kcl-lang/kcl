use std::fs;

use crate::*;
use glob::glob;

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_file_read(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(x) = args.arg_i_str(0, None) {
        let contents = fs::read_to_string(&x)
            .unwrap_or_else(|e| panic!("failed to access the file '{}': {}", x, e.to_string()));

        let s = ValueRef::str(contents.as_ref());
        return s.into_raw(ctx);
    }

    panic!("read() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_file_glob(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let ctx = mut_ptr_as_ref(ctx);

    let pattern = args
        .arg_i_str(0, None)
        .expect("glob() takes exactly one argument (0 given)");

    let mut matched_paths = vec![];
    for entry in
        glob(&pattern).unwrap_or_else(|e| panic!("Failed to read glob pattern: {}", e.to_string()))
    {
        match entry {
            Ok(path) => matched_paths.push(path.display().to_string()),
            Err(e) => panic!(
                "failed to access the file matching '{}': {}",
                pattern,
                e.to_string()
            ),
        }
    }

    return ValueRef::list_str(matched_paths.as_slice()).into_raw(ctx);
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_file_modpath(
    ctx: *mut kclvm_context_t,
    _args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let s = ValueRef::str(&ctx.module_path.as_ref());
    return s.into_raw(ctx);
}
