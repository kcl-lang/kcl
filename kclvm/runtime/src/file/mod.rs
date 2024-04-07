use std::fs;

use crate::*;
use glob::glob;
use std::path::Path;

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_file_read(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(x) = get_call_arg_str(args, kwargs, 0, Some("filepath")) {
        let contents = fs::read_to_string(&x)
            .unwrap_or_else(|e| panic!("failed to access the file '{}': {}", x, e));

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
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    let pattern = get_call_arg_str(args, kwargs, 0, Some("pattern"))
        .expect("glob() takes exactly one argument (0 given)");

    let mut matched_paths = vec![];
    for entry in glob(&pattern).unwrap_or_else(|e| panic!("Failed to read glob pattern: {}", e)) {
        match entry {
            Ok(path) => matched_paths.push(path.display().to_string()),
            Err(e) => panic!("failed to access the file matching '{}': {}", pattern, e),
        }
    }

    ValueRef::list_str(matched_paths.as_slice()).into_raw(ctx)
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_file_modpath(
    ctx: *mut kclvm_context_t,
    _args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let s = ValueRef::str(ctx.module_path.as_ref());
    s.into_raw(ctx)
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_file_workdir(
    ctx: *mut kclvm_context_t,
    _args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let s = ValueRef::str(ctx.workdir.as_ref());
    s.into_raw(ctx)
}

/// Whether this file path exists. Returns true if the path points at
/// an existing entity. This function will traverse symbolic links to
/// query information about the destination file.
#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_file_exists(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(path) = get_call_arg_str(args, kwargs, 0, Some("filepath")) {
        let exist = Path::new(&path).exists();
        return ValueRef::bool(exist).into_raw(ctx);
    }

    panic!("read() takes exactly one argument (0 given)");
}

/// Returns the canonical, absolute form of the path with all intermediate
/// components normalized and symbolic links resolved.
#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_file_abs(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(path) = get_call_arg_str(args, kwargs, 0, Some("filepath")) {
        if let Ok(abs_path) = Path::new(&path).canonicalize() {
            return ValueRef::str(abs_path.to_str().unwrap()).into_raw(ctx);
        } else {
            panic!("Could not get the absolute path of {path}");
        }
    }

    panic!("read() takes exactly one argument (0 given)");
}
