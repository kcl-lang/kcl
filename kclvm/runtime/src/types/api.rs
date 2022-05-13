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
// new
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_Any() -> *mut kclvm_type_t {
    new_mut_ptr(Type::any())
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_Bool() -> *mut kclvm_type_t {
    new_mut_ptr(Type::bool())
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_BoolLit(v: kclvm_bool_t) -> *mut kclvm_type_t {
    new_mut_ptr(Type::bool_lit(v != 0))
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_Int() -> *mut kclvm_type_t {
    new_mut_ptr(Type::int())
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_IntLit(v: i64) -> *mut kclvm_type_t {
    new_mut_ptr(Type::int_lit(v))
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_Float() -> *mut kclvm_type_t {
    new_mut_ptr(Type::float())
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_FloatLit(v: f64) -> *mut kclvm_type_t {
    new_mut_ptr(Type::float_lit(v))
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_Str() -> *mut kclvm_type_t {
    new_mut_ptr(Type::str())
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_StrLit(s: *const kclvm_char_t) -> *mut kclvm_type_t {
    return new_mut_ptr(Type::str_lit(c2str(s)));
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_List(elem_type: *const kclvm_type_t) -> *mut kclvm_type_t {
    return new_mut_ptr(Type::list(ptr_as_ref(elem_type)));
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_Dict(
    key_type: *const kclvm_type_t,
    elem_type: *const kclvm_type_t,
) -> *mut kclvm_type_t {
    return new_mut_ptr(Type::dict(ptr_as_ref(key_type), ptr_as_ref(elem_type)));
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_Union(
    n: kclvm_size_t,
    elem_types: *const *const kclvm_type_t,
) -> *mut kclvm_type_t {
    unsafe {
        let mut ut: UnionType = Default::default();

        let _ = std::slice::from_raw_parts(elem_types, n as usize)
            .iter()
            .map(|arg| ut.elem_types.push(ptr_as_ref(*arg).clone()));

        new_mut_ptr(Type::union_type(ut))
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_Schema(
    name: *const kclvm_char_t,
    parent_name: *const kclvm_char_t,
    _is_relaxed: kclvm_bool_t,
    field_num: kclvm_size_t,
    field_names: *const *const kclvm_char_t,
    field_types: *const *const kclvm_type_t,
) -> *mut kclvm_type_t {
    unsafe {
        let mut st: SchemaType = SchemaType {
            name: c2str(name).to_string(),
            parent_name: c2str(parent_name).to_string(),
            ..Default::default()
        };

        let _ = std::slice::from_raw_parts(field_names, field_num as usize)
            .iter()
            .map(|arg| st.field_names.push(c2str(*arg).to_string()));
        let _ = std::slice::from_raw_parts(field_types, field_num as usize)
            .iter()
            .map(|arg| st.field_types.push(ptr_as_ref(*arg).clone()));

        new_mut_ptr(Type::schema_type(st))
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_type_Func(
    args_len: kclvm_size_t,
    args_types: *const *const kclvm_type_t,
    return_type: *const kclvm_type_t,
) -> *mut kclvm_type_t {
    unsafe {
        let mut ft: FuncType = FuncType {
            return_type: Box::new(ptr_as_ref(return_type).clone()),
            ..Default::default()
        };
        let _ = std::slice::from_raw_parts(args_types, args_len as usize)
            .iter()
            .map(|arg| ft.args_types.push(ptr_as_ref(*arg).clone()));

        new_mut_ptr(Type::func_type(ft))
    }
}

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
