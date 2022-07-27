use protobuf::Message;

use crate::model::gpyrpc::*;
use crate::service::service::KclvmService;
use std::ffi::CString;
use std::os::raw::c_char;

pub fn _kclvm_get_service_fn_ptr_by_name(name: &str) -> u64 {
    match name {
        "KclvmService.Ping" => ping as *const () as u64,
        "KclvmService.ExecProgram" => exec_program as *const () as u64,
        _ => panic!("unknown method name : {}", name),
    }
}

pub fn ping(serv: &mut KclvmService, args: &[u8]) -> *const c_char {
    let args = Ping_Args::parse_from_bytes(args).unwrap();
    let res = serv.ping(&args);
    CString::new(res.write_to_bytes().unwrap())
        .unwrap()
        .into_raw()
}

pub fn exec_program(serv: &mut KclvmService, args: &[u8]) -> *const c_char {
    let args = ExecProgram_Args::parse_from_bytes(args).unwrap();
    let res = serv.exec_program(&args);
    CString::new(res.write_to_bytes().unwrap())
        .unwrap()
        .into_raw()
}
