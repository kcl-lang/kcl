//! Copyright The KCL Authors. All rights reserved.
#![allow(clippy::missing_safety_doc)]

use std::os::raw::c_char;

use crate::*;

#[allow(non_camel_case_types)]
type kclvm_value_ref_t = ValueRef;

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_option_init(
    ctx: *mut kclvm_context_t,
    key: *const c_char,
    value: *const c_char,
) {
    let ctx = mut_ptr_as_ref(ctx);
    ctx.builtin_option_init(c2str(key), c2str(value));
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_option_reset(
    ctx: *mut kclvm_context_t,
    _args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let ctx_ref = mut_ptr_as_ref(ctx);

    ctx_ref.builtin_option_reset();
    kclvm_value_Undefined(ctx)
}

// def kcl_option(name: str, *, type="", required=False, default=None, help="", file="", line=0) -> typing.Any:

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_option(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let ctx_ref = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    fn _value_to_type(this: &ValueRef, typ: String) -> ValueRef {
        if typ.is_empty() {
            return this.clone();
        }
        if typ == "bool" {
            match *this.rc.borrow() {
                Value::bool_value(ref v) => {
                    return ValueRef::bool(*v);
                }
                Value::int_value(ref v) => {
                    return ValueRef::bool(*v != 0);
                }
                Value::float_value(ref v) => {
                    return ValueRef::bool(*v != 0.0);
                }
                Value::str_value(ref v) => {
                    return ValueRef::bool(v == "True" || v == "true");
                }
                _ => {
                    return ValueRef::undefined();
                }
            }
        }
        if typ == "int" {
            match *this.rc.borrow() {
                Value::bool_value(ref v) => {
                    if *v {
                        return ValueRef::int(1);
                    } else {
                        return ValueRef::int(0);
                    }
                }
                Value::int_value(ref v) => {
                    return ValueRef::int(*v);
                }
                Value::float_value(ref v) => {
                    return ValueRef::int(*v as i64);
                }
                Value::str_value(ref v) => {
                    match v.parse::<i64>() {
                        Ok(n) => return ValueRef::int(n),
                        _ => panic!("cannot use '{v}' as type '{typ}'"),
                    };
                }
                _ => {
                    let err_msg = format!("cannot use '{this}' as type '{typ}'");
                    panic!("{}", err_msg);
                }
            }
        }
        if typ == "float" {
            match *this.rc.borrow() {
                Value::bool_value(ref v) => {
                    if *v {
                        return ValueRef::float(1.0);
                    } else {
                        return ValueRef::float(0.0);
                    }
                }
                Value::int_value(ref v) => {
                    return ValueRef::float(*v as f64);
                }
                Value::float_value(ref v) => {
                    return ValueRef::float(*v);
                }
                Value::str_value(ref v) => {
                    match v.parse::<f64>() {
                        Ok(n) => return ValueRef::float(n),
                        _ => return ValueRef::float(0.0),
                    };
                }
                _ => {
                    let err_msg = format!("cannot use '{this}' as type '{typ}'");
                    panic!("{}", err_msg);
                }
            }
        }
        if typ == "str" {
            match *this.rc.borrow() {
                Value::bool_value(ref v) => {
                    let s = format!("{}", *v);
                    return ValueRef::str(s.as_ref());
                }
                Value::int_value(ref v) => {
                    let s = format!("{}", *v);
                    return ValueRef::str(s.as_ref());
                }
                Value::float_value(ref v) => {
                    let s = format!("{}", *v);
                    return ValueRef::str(s.as_ref());
                }
                Value::str_value(ref v) => {
                    return ValueRef::str(v.as_ref());
                }
                _ => {
                    let err_msg = format!("cannot use '{this}' as type '{typ}'");
                    panic!("{}", err_msg);
                }
            }
        }
        if typ == "list" {
            match *this.rc.borrow() {
                Value::list_value(_) => {
                    return this.clone();
                }
                _ => {
                    let err_msg = format!("cannot use '{this}' as type '{typ}'");
                    panic!("{}", err_msg);
                }
            }
        }
        if typ == "dict" {
            match *this.rc.borrow() {
                Value::dict_value(_) => {
                    return this.clone();
                }
                _ => {
                    let err_msg = format!("cannot use '{this}' as type '{typ}'");
                    panic!("{}", err_msg);
                }
            }
        }

        panic!("unknown type '{typ}'");
    }

    if let Some(arg0) = get_call_arg_str(args, kwargs, 0, Some("key")) {
        if let Some(x) = ctx_ref.app_args.get(&arg0) {
            if *x == 0 {
                return kclvm_value_Undefined(ctx);
            }

            let opt_value = mut_ptr_as_ref((*x) as *mut kclvm_value_ref_t);

            if let Some(kwarg_type) = get_call_arg_str(args, kwargs, 1, Some("type")) {
                return _value_to_type(opt_value, kwarg_type).into_raw(ctx_ref);
            }

            return (*x) as *mut kclvm_value_ref_t;
        } else if let Some(kwarg_default) = get_call_arg(args, kwargs, 3, Some("default")) {
            if let Some(kwarg_type) = get_call_arg_str(args, kwargs, 1, Some("type")) {
                return _value_to_type(&kwarg_default, kwarg_type).into_raw(ctx_ref);
            }

            return kwarg_default.into_raw(ctx_ref);
        }
    }

    let required = get_call_arg_bool(args, kwargs, 2, Some("required")).unwrap_or_default();
    if required {
        let name = args.arg_i_str(0, Some("?".to_string())).unwrap();
        panic!("option('{name}') must be initialized, try '-D {name}=?' argument");
    }

    kclvm_value_None(ctx)
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_print(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx_ref = mut_ptr_as_ref(ctx);
    // args
    let list = args.as_list_ref();
    let values: Vec<String> = list.values.iter().map(|v| v.to_string()).collect();
    ctx_ref.log_message.push_str(&values.join(" "));
    let dict = kwargs.as_dict_ref();
    // kwargs: end
    if let Some(c) = dict.values.get("end") {
        ctx_ref.log_message.push_str(&format!("{c}"));
    } else {
        ctx_ref.log_message.push('\n');
    }
    kclvm_value_None(ctx)
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_len(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(arg) = get_call_arg(args, kwargs, 0, Some("inval")) {
        return kclvm_value_Int(ctx, arg.len() as i64);
    }
    panic!("len() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_any_true(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("inval")) {
        return kclvm_value_Bool(ctx, arg0.any_true() as i8);
    }
    kclvm_value_Bool(ctx, 0)
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_isunique(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("inval")) {
        return kclvm_value_Bool(ctx, arg0.isunique() as i8);
    }
    kclvm_value_Bool(ctx, 0)
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_sorted(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("inval")) {
        let reverse = get_call_arg(args, kwargs, 1, Some("reverse"));
        return arg0.sorted(reverse.as_ref()).into_raw(ctx);
    }
    panic!("sorted() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_int(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("number")) {
        let base = get_call_arg(args, kwargs, 1, Some("base"));
        return arg0.convert_to_int(ctx, base.as_ref()).into_raw(ctx);
    }
    panic!("int() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_float(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("number")) {
        return arg0.convert_to_float(ctx).into_raw(ctx);
    }
    panic!("float() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_bool(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("x")) {
        return ValueRef::bool(arg0.is_truthy()).into_raw(ctx);
    }
    panic!("bool() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("x")) {
        return ValueRef::str(&arg0.to_string()).into_raw(ctx);
    }
    ValueRef::str("").into_raw(ctx)
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_max(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    if args.args_len() > 1 {
        return args.max_value().into_raw(ctx);
    }
    if let Some(arg0) = args.arg_0() {
        return arg0.max_value().into_raw(ctx);
    }
    panic!("max() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_min(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    if args.args_len() > 1 {
        return args.min_value().into_raw(ctx);
    }
    if let Some(arg0) = args.arg_0() {
        return arg0.min_value().into_raw(ctx);
    }
    panic!("min() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_multiplyof(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let (Some(arg0), Some(arg1)) = (
        get_call_arg(args, kwargs, 0, Some("a")),
        get_call_arg(args, kwargs, 1, Some("b")),
    ) {
        return builtin::multiplyof(&arg0, &arg1).into_raw(ctx);
    }
    panic!(
        "multiplyof() takes exactly two argument ({} given)",
        args.args_len()
    );
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_abs(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("inval")) {
        return arg0.abs().into_raw(ctx);
    }
    panic!("abs() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_all_true(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("inval")) {
        return kclvm_value_Bool(ctx, arg0.all_true() as i8);
    }
    kclvm_value_Bool(ctx, 0)
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_hex(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("number")) {
        return arg0.hex().into_raw(ctx);
    }
    panic!("hex() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_sum(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx_ref = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    match get_call_arg(args, kwargs, 0, Some("iterable")) {
        Some(arg0) => match get_call_arg(args, kwargs, 1, Some("start")) {
            Some(arg1) => arg0.sum(ctx_ref, &arg1).into_raw(ctx_ref),
            _ => arg0.sum(ctx_ref, &ValueRef::int(0)).into_raw(ctx_ref),
        },
        _ => kclvm_value_Undefined(ctx),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_pow(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx_ref = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    match (
        get_call_arg(args, kwargs, 0, Some("x")),
        get_call_arg(args, kwargs, 1, Some("y")),
    ) {
        (Some(arg0), Some(arg1)) => match get_call_arg(args, kwargs, 2, Some("z")) {
            Some(arg2) => builtin::pow(&arg0, &arg1, &arg2).into_raw(ctx_ref),
            _ => builtin::pow(&arg0, &arg1, &ValueRef::none()).into_raw(ctx_ref),
        },
        _ => kclvm_value_Undefined(ctx),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_round(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx_ref = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    match get_call_arg(args, kwargs, 0, Some("number")) {
        Some(arg0) => match get_call_arg(args, kwargs, 1, Some("ndigits")) {
            Some(arg1) => builtin::round(&arg0, &arg1).into_raw(ctx_ref),
            _ => builtin::round(&arg0, &ValueRef::none()).into_raw(ctx_ref),
        },
        _ => kclvm_value_Undefined(ctx),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_zip(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    args.zip().into_raw(ctx)
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_list(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if args.args_len() > 0 {
        if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("x")) {
            return builtin::list(Some(&arg0)).into_raw(ctx);
        }
        panic!("invalid arguments in list() function");
    } else {
        builtin::list(None).into_raw(ctx)
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_dict(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let mut dict = ValueRef::dict(None);
    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("x")) {
        let v = builtin::dict(ctx, Some(&arg0));
        dict.dict_insert_unpack(ctx, &v);
    }
    let v = builtin::dict(ctx, Some(kwargs));
    dict.dict_insert_unpack(ctx, &v);
    dict.into_raw(ctx)
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_typeof(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("x")) {
        if let Some(full_name) = get_call_arg(args, kwargs, 1, Some("full_name")) {
            return builtin::type_of(&arg0, &full_name).into_raw(ctx);
        }
        return builtin::type_of(&arg0, &ValueRef::bool(false)).into_raw(ctx);
    }

    panic!("typeof() missing 1 required positional argument: 'x'");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_bin(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("number")) {
        return arg0.bin().into_raw(ctx);
    }
    panic!("bin() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_oct(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("number")) {
        return arg0.oct().into_raw(ctx);
    }
    panic!("oct() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_ord(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("c")) {
        return arg0.ord().into_raw(ctx);
    }
    panic!("ord() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_range(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let ctx_ref = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    match get_call_arg(args, kwargs, 0, Some("start")) {
        Some(arg0) => match get_call_arg(args, kwargs, 1, Some("stop")) {
            Some(arg1) => match get_call_arg(args, kwargs, 2, Some("step")) {
                Some(arg2) => builtin::range(&arg0, &arg1, &arg2).into_raw(ctx_ref),
                _ => builtin::range(&arg0, &arg1, &ValueRef::int(1)).into_raw(ctx_ref),
            },
            _ => builtin::range(&ValueRef::int(0), &arg0, &ValueRef::int(1)).into_raw(ctx_ref),
        },
        _ => kclvm_value_Undefined(ctx),
    }
}
