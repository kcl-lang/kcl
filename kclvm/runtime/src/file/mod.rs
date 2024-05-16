mod utils;

use std::{fs, io::ErrorKind};

use crate::*;
use glob::glob;
use std::io::Write;
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

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_file_mkdir(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(path) = get_call_arg_str(args, kwargs, 0, Some("directory")) {
        let exists = get_call_arg_bool(args, kwargs, 1, Some("exists")).unwrap_or_default();
        if let Err(e) = fs::create_dir_all(&path) {
            // Ignore the file exists error.
            if exists && matches!(e.kind(), ErrorKind::AlreadyExists) {
                return ValueRef::none().into_raw(ctx);
            }
            panic!("Failed to create directory '{}': {}", path, e);
        }
        return ValueRef::none().into_raw(ctx);
    }

    panic!("mkdir() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_file_delete(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(path) = get_call_arg_str(args, kwargs, 0, Some("filepath")) {
        if let Err(e) = fs::remove_file(&path) {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    // if file not found, try to remove it as a directory
                    if let Err(e) = fs::remove_dir(&path) {
                        panic!("failed to delete '{}': {}", path, e);
                    }
                }
                _ => {
                    panic!("failed to delete '{}': {}", path, e);
                }
            }
        }
        return ValueRef::none().into_raw(ctx);
    }

    panic!("delete() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_file_cp(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(src_path) = get_call_arg_str(args, kwargs, 0, Some("src")) {
        if let Some(dest_path) = get_call_arg_str(args, kwargs, 1, Some("dest")) {
            let src_path = Path::new(&src_path);
            let dest_path = Path::new(&dest_path);
            let result = if src_path.is_dir() {
                utils::copy_directory(&src_path, &dest_path)
            } else {
                fs::copy(&src_path, &dest_path).map(|_| ())
            };
            if let Err(e) = result {
                panic!(
                    "Failed to copy from '{}' to '{}': {}",
                    src_path.display(),
                    dest_path.display(),
                    e
                );
            }
            return ValueRef::none().into_raw(ctx);
        } else {
            panic!("cp() missing 'dest_path' argument");
        }
    } else {
        panic!("cp() missing 'src_path' argument");
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_file_mv(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(src_path) = get_call_arg_str(args, kwargs, 0, Some("src")) {
        if let Some(dest_path) = get_call_arg_str(args, kwargs, 1, Some("dest")) {
            if let Err(e) = fs::rename(&src_path, &dest_path) {
                panic!("Failed to move '{}' to '{}': {}", src_path, dest_path, e);
            }
            return ValueRef::none().into_raw(ctx);
        } else {
            panic!("mv() missing 'dest_path' argument");
        }
    } else {
        panic!("mv() missing 'src_path' argument");
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_file_size(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(path) = get_call_arg_str(args, kwargs, 0, Some("filepath")) {
        let metadata = fs::metadata(&path);
        match metadata {
            Ok(metadata) => {
                let size = metadata.len();
                let value = kclvm::ValueRef::int(size as i64);
                return value.into_raw(ctx);
            }
            Err(e) => {
                panic!("failed to get size of '{}': {}", path, e);
            }
        }
    }

    panic!("size() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_file_write(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(path) = get_call_arg_str(args, kwargs, 0, Some("filepath")) {
        if let Some(content) = get_call_arg_str(args, kwargs, 1, Some("content")) {
            match fs::File::create(&path) {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(content.as_bytes()) {
                        panic!("Failed to write to '{}': {}", path, e);
                    }
                    return ValueRef::none().into_raw(ctx);
                }
                Err(e) => panic!("Failed to create file '{}': {}", path, e),
            }
        } else {
            panic!("write() missing 'content' argument");
        }
    } else {
        panic!("write() missing 'filepath' argument");
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_file_append(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(path) = get_call_arg_str(args, kwargs, 0, Some("filepath")) {
        if let Some(content) = get_call_arg_str(args, kwargs, 1, Some("content")) {
            // Open the file in append mode, creating it if it doesn't exist
            match fs::OpenOptions::new().append(true).create(true).open(&path) {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(content.as_bytes()) {
                        panic!("Failed to append to file '{}': {}", path, e);
                    }
                    return ValueRef::none().into_raw(ctx);
                }
                Err(e) => {
                    panic!("Failed to open or create file '{}': {}", path, e);
                }
            }
        } else {
            panic!("append() requires 'content' argument");
        }
    } else {
        panic!("append() requires 'filepath' argument");
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_file_read_env(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(key) = get_call_arg_str(args, kwargs, 0, Some("key")) {
        match std::env::var(key) {
            Ok(v) => ValueRef::str(&v).into_raw(ctx),
            Err(_) => ValueRef::undefined().into_raw(ctx),
        }
    } else {
        panic!("read_env() requires 'key' argument");
    }
}
