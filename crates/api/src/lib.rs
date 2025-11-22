//! # KCL Rust SDK
//!
//! ## How to Use
//!
//! ```no_check,no_run
//! cargo add --git https://github.com/kcl-lang/kcl kclvm_api
//! ```
//!
//! Write the Code
//!
//! ```no_run
//! use kclvm_api::*;
//! use std::path::Path;
//! use anyhow::Result;
//!
//! fn main() -> Result<()> {
//!     let api = API::default();
//!     let args = &ExecProgramArgs {
//!         work_dir: Path::new(".").join("testdata").canonicalize().unwrap().display().to_string(),
//!         k_filename_list: vec!["test.k".to_string()],
//!         ..Default::default()
//!     };
//!     let exec_result = api.exec_program(args)?;
//!     assert_eq!(exec_result.yaml_result, "alice:\n  age: 18");
//!     Ok(())
//! }
//! ```
#[cfg(test)]
pub mod capi_test;
pub mod service;

pub mod gpyrpc {
    include!(concat!(env!("OUT_DIR"), "/gpyrpc.rs"));
}

pub use crate::gpyrpc::*;
use crate::service::capi::{kclvm_service_call_with_length, kclvm_service_new};
use crate::service::service_impl::KclvmServiceImpl;
use anyhow::Result;
use std::ffi::{c_char, CString};

pub type API = KclvmServiceImpl;

/// Call KCL API with the API name and argument protobuf bytes.
#[inline]
pub fn call<'a>(name: &'a [u8], args: &'a [u8]) -> Result<Vec<u8>> {
    call_with_plugin_agent(name, args, 0)
}

/// Call KCL API with the API name, argument protobuf bytes and the plugin agent pointer address.
pub fn call_with_plugin_agent<'a>(
    name: &'a [u8],
    args: &'a [u8],
    plugin_agent: u64,
) -> Result<Vec<u8>> {
    let mut result_len: usize = 0;
    let result_ptr = {
        let serv = kclvm_service_new(plugin_agent);
        let args_len = args.len();
        let name = unsafe { CString::from_vec_unchecked(name.to_vec()) };
        let args = unsafe { CString::from_vec_unchecked(args.to_vec()) };
        kclvm_service_call_with_length(
            serv,
            name.as_ptr(),
            args.as_ptr() as *const c_char,
            args_len,
            &mut result_len,
        )
    };
    let result = unsafe {
        let mut dest_data: Vec<u8> = Vec::with_capacity(result_len);
        let dest_ptr: *mut u8 = dest_data.as_mut_ptr();
        std::ptr::copy_nonoverlapping(result_ptr as *const u8, dest_ptr, result_len);
        dest_data.set_len(result_len);
        dest_data
    };

    Ok(result)
}

/// call_native is a universal KCL API interface that is consistent with the methods and parameters defined in Protobuf.
/// The first two parameters represent the name and length of the calling method, the middle two parameters represent
/// the Protobuf byte sequence and length of the calling parameter, and the return parameter is the byte sequence and
/// length of Protobuf.
#[no_mangle]
pub extern "C-unwind" fn call_native(
    name_ptr: *const u8,
    name_len: usize,
    args_ptr: *const u8,
    args_len: usize,
    result_ptr: *mut u8,
) -> usize {
    let name = unsafe { std::slice::from_raw_parts(name_ptr, name_len) };
    let args = unsafe { std::slice::from_raw_parts(args_ptr, args_len) };
    let res = call(name, args);
    let result = match res {
        Ok(res) => res,
        Err(err) => err.to_string().into_bytes(),
    };
    unsafe {
        std::ptr::copy_nonoverlapping(result.as_ptr(), result_ptr, result.len());
    }
    result.len()
}
