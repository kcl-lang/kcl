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
use std::ffi::CString;

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
        let args = CString::new(args)?;
        let call = CString::new(name)?;
        let serv = kclvm_service_new(plugin_agent);
        kclvm_service_call_with_length(serv, call.as_ptr(), args.as_ptr(), &mut result_len)
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
