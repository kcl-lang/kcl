//! The `kclvm` command-line interface.

use std::{
    ffi::{CString, c_char, c_int},
    process::ExitCode,
};

#[link(name = "kcl")]
#[allow(improper_ctypes)]
unsafe extern "C-unwind" {
    unsafe fn libkcl_main(argc: c_int, argv: *const *const c_char) -> *mut ExitCode;
}

fn main() -> ExitCode {
    // create a vector of zero terminated strings
    let args = std::env::args()
        .map(|arg| CString::new(arg).unwrap())
        .collect::<Vec<CString>>();
    // convert the strings to raw pointers
    let c_args = args
        .iter()
        .map(|arg| arg.as_ptr())
        .collect::<Vec<*const c_char>>();
    unsafe {
        // pass the pointer of the vector's internal buffer to a C function
        let result = libkcl_main(c_args.len() as c_int, c_args.as_ptr());
        let result = Box::from_raw(result);
        *result
    }
}
