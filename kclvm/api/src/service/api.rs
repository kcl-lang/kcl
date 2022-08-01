use protobuf::Message;

use crate::model::gpyrpc::*;
use crate::service::service::KclvmService;
use kclvm::utils::*;
use std::ffi::CString;
use std::os::raw::c_char;

pub fn _kclvm_get_service_fn_ptr_by_name(name: &str) -> u64 {
    match name {
        "KclvmService.Ping" => ping as *const () as u64,
        "KclvmService.ExecProgram" => exec_program as *const () as u64,
        _ => panic!("unknown method name : {}", name),
    }
}

///ping is used to test whether kclvm service is successfully imported
///arguments and return results should be consistent
pub fn ping(serv: *mut KclvmService, args: &[u8]) -> *const c_char {
    let serv_ref = mut_ptr_as_ref(serv);
    let args = Ping_Args::parse_from_bytes(args).unwrap();
    let res = serv_ref.ping(&args);
    CString::new(res.write_to_bytes().unwrap())
        .unwrap()
        .into_raw()
}

/// exec_program provides users with the ability to execute KCL code
///
/// # Parameters
///
/// `serv`: [*mut kclvm_service]
///     The pointer of &\[[KclvmService]]
///
///
/// `args`: [&[u8]]
///     the items and compile parameters selected by the user in the KCLVM CLI
///     serialized as protobuf byte sequence
///
/// # Returns
///
/// result: [*const c_char]
///     Result of the call serialized as protobuf byte sequence
pub fn exec_program(serv: &mut KclvmService, args: &[u8]) -> *const c_char {
    let serv_ref = mut_ptr_as_ref(serv);
    let args = ExecProgram_Args::parse_from_bytes(args).unwrap();
    let res = serv_ref.exec_program(&args);
    let result_byte = match res {
        Ok(res) => match res.write_to_bytes() {
            Ok(bytes) => bytes,
            Err(err) => panic!("{}", err.to_string()),
        },
        Err(err) => panic!("{}", err.clone()),
    };
    CString::new(result_byte).unwrap().into_raw()
}
