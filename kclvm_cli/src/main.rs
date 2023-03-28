//! The `kclvm` command-line interface.

use std::ffi::{c_char, c_int, CString};

#[link(name = "kclvm_cli_cdylib")]
extern "C" {
    fn kclvm_cli_main(argc: c_int, argv: *const *const c_char) -> *mut c_char;
}

fn main() {
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
        let result = CString::from_raw(kclvm_cli_main(c_args.len() as c_int, c_args.as_ptr()));
        let result_str = result.to_str().unwrap();
        if !result_str.is_empty() {
            println!("{}", result_str);
        }
    }
}
