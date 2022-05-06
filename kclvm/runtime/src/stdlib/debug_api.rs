// Copyright 2021 The KCL Authors. All rights reserved.

use crate::*;

#[allow(non_camel_case_types)]
type kclvm_type_t = Type;

#[allow(non_camel_case_types)]
type kclvm_value_ref_t = ValueRef;

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_debug_hello() {
    println!("kclvm_debug_hello: hello kclvm")
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_debug_print(cs: *const i8) {
    let msg = unsafe { std::ffi::CStr::from_ptr(cs) }.to_str().unwrap();
    println!("kclvm_debug_print: {}", msg)
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_debug_print_str_list(len: i32, ss: &mut &mut i8) {
    let x = crate::convert_double_pointer_to_vec(ss, len as usize);
    println!("kclvm_debug_print_str_list: {:?}", x);
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_debug_print_type(p: *const kclvm_type_t) {
    let p = ptr_as_ref(p);
    println!("kclvm_debug_print_type: {:?}", p)
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_debug_print_value(p: *const kclvm_value_ref_t) {
    let p = ptr_as_ref(p);
    println!("kclvm_debug_print_value: {:?}", p);
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_debug_print_value_json_string(p: *const kclvm_value_ref_t) {
    let p = ptr_as_ref(p);
    println!("kclvm_debug_print_value: {}", p.to_json_string());
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_debug_invoke_func(fn_ptr: extern "C" fn()) {
    println!("kclvm_debug_invoke_func begin");
    fn_ptr();
    println!("kclvm_debug_invoke_func end");
}
