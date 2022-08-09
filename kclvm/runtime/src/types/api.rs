// Copyright 2021 The KCL Authors. All rights reserved.

use crate::*;

#[allow(dead_code, non_camel_case_types)]
type kclvm_context_t = Context;

#[allow(dead_code, non_camel_case_types)]
type kclvm_kind_t = Kind;

#[allow(dead_code, non_camel_case_types)]
type kclvm_type_t = Type;

#[allow(dead_code, non_camel_case_types)]
type kclvm_value_t = Value;

#[allow(dead_code, non_camel_case_types)]
type kclvm_char_t = i8;

#[allow(dead_code, non_camel_case_types)]
type kclvm_size_t = i32;

#[allow(dead_code, non_camel_case_types)]
type kclvm_bool_t = i8;

#[allow(dead_code, non_camel_case_types)]
type kclvm_int_t = i64;

#[allow(dead_code, non_camel_case_types)]
type kclvm_float_t = f64;

// ----------------------------------------------------------------------------
// delete
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_delete(p: *mut kclvm_type_t) {
    free_mut_ptr(p);
}

// ----------------------------------------------------------------------------
// kind
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_kind(p: *const kclvm_type_t) -> kclvm_kind_t {
    let p = ptr_as_ref(p);

    p.kind()
}

// ----------------------------------------------------------------------------
// type_str
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_str(p: *const kclvm_type_t) -> kclvm_kind_t {
    let p = ptr_as_ref(p);

    p.kind()
}

// ----------------------------------------------------------------------------
// lit value
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_BoolLit_value(p: *const kclvm_type_t) -> kclvm_bool_t {
    match ptr_as_ref(p) {
        Type::bool_lit_type(ref v) => *v as kclvm_bool_t,
        _ => 0,
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_IntLit_value(p: *const kclvm_type_t) -> i64 {
    let p = ptr_as_ref(p);
    match p {
        Type::int_lit_type(ref v) => *v,
        _ => 0,
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_FloatLit_value(p: *const kclvm_type_t) -> f64 {
    let p = ptr_as_ref(p);
    match p {
        Type::float_lit_type(ref v) => *v,
        _ => 0.0,
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_StrLit_value(p: *const kclvm_type_t) -> *const kclvm_char_t {
    let p = ptr_as_ref(p);
    match p {
        Type::str_lit_type(ref v) => v.as_ptr() as *const kclvm_char_t,
        _ => std::ptr::null(),
    }
}

// ----------------------------------------------------------------------------
// list/dict type
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_key_type(p: *const kclvm_type_t) -> *const kclvm_type_t {
    let p = ptr_as_ref(p);
    match p {
        Type::dict_type(ref v) => {
            return v.key_type.as_ref() as *const Type;
        }
        _ => std::ptr::null(),
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_elem_type(p: *const kclvm_type_t) -> *const kclvm_type_t {
    let p = ptr_as_ref(p);
    match p {
        Type::list_type(ref v) => {
            return v.elem_type.as_ref() as *const Type;
        }
        Type::dict_type(ref v) => {
            return v.elem_type.as_ref() as *const Type;
        }
        _ => std::ptr::null(),
    }
}

// ----------------------------------------------------------------------------
// schema
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_schema_name(p: *const kclvm_type_t) -> *const kclvm_char_t {
    let p = ptr_as_ref(p);
    match p {
        Type::schema_type(ref v) => v.name.as_ptr() as *const kclvm_char_t,
        _ => std::ptr::null(),
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_schema_parent_name(p: *const kclvm_type_t) -> *const kclvm_char_t {
    let p = ptr_as_ref(p);
    match p {
        Type::schema_type(ref v) => v.parent_name.as_ptr() as *const kclvm_char_t,
        _ => std::ptr::null(),
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_schema_relaxed(p: *const kclvm_type_t) -> kclvm_bool_t {
    let p = ptr_as_ref(p);
    match p {
        Type::schema_type(..) => false as kclvm_bool_t,
        _ => 0,
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_schema_field_num(p: *const kclvm_type_t) -> kclvm_size_t {
    let p = ptr_as_ref(p);
    match p {
        Type::schema_type(ref v) => v.field_names.len() as kclvm_size_t,
        _ => 0,
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_schema_field_name(
    p: *const kclvm_type_t,
    i: kclvm_size_t,
) -> *const kclvm_char_t {
    let p = ptr_as_ref(p);
    match p {
        Type::schema_type(ref v) => v.field_names[i as usize].as_ptr() as *const kclvm_char_t,
        _ => std::ptr::null(),
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_schema_field_type(
    p: *const kclvm_type_t,
    i: kclvm_size_t,
) -> *const kclvm_type_t {
    let p = ptr_as_ref(p);
    match p {
        Type::schema_type(ref v) => &v.field_types[i as usize] as *const kclvm_type_t,
        _ => std::ptr::null(),
    }
}

// ----------------------------------------------------------------------------
// func (for plugin)
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_arg_num(p: *const kclvm_type_t) -> kclvm_size_t {
    let p = ptr_as_ref(p);
    match p {
        Type::func_type(ref v) => v.args_types.len() as kclvm_size_t,
        _ => 0,
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_arg_type(
    p: *const kclvm_type_t,
    i: kclvm_size_t,
) -> *const kclvm_type_t {
    let p = ptr_as_ref(p);
    match p {
        Type::func_type(ref v) => &v.args_types[i as usize] as *const kclvm_type_t,
        _ => std::ptr::null(),
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_return_type(p: *const kclvm_type_t) -> *const kclvm_type_t {
    let p = ptr_as_ref(p);
    match p {
        Type::func_type(ref v) => v.return_type.as_ref() as *const kclvm_type_t,
        _ => std::ptr::null(),
    }
}

// ----------------------------------------------------------------------------
// END
// ----------------------------------------------------------------------------
