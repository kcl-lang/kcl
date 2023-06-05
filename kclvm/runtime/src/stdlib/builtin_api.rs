// Copyright 2021 The KCL Authors. All rights reserved.
#![allow(clippy::missing_safety_doc)]

use crate::*;

#[allow(non_camel_case_types)]
type kclvm_value_ref_t = ValueRef;

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_option_init(
    ctx: *mut kclvm_context_t,
    key: *const i8,
    value: *const i8,
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
    let ctx = mut_ptr_as_ref(ctx);

    ctx.builtin_option_reset();
    kclvm_value_Undefined()
}

// def kcl_option(name: str, *, type="", required=False, default=None, help="", file="", line=0) -> typing.Any:

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_option(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    let mut list_option_mode = false;

    if ctx.cfg.list_option_mode {
        list_option_mode = true;

        let name = args.arg_i_str(0, Some("?".to_string())).unwrap();
        let typ = kwargs.kwarg_str("type", Some("".to_string())).unwrap();
        let required = kwargs.kwarg_bool("required", Some(false)).unwrap();
        let help = kwargs.kwarg_str("help", Some("".to_string())).unwrap();

        let mut default_value: Option<String> = None;
        if let Some(x) = kwargs.kwarg("default") {
            default_value = Some(x.to_string());
        }

        ctx.define_option(
            name.as_str(),
            typ.as_str(),
            required,
            default_value,
            help.as_str(),
        );
    }

    fn _value_to_type(this: &ValueRef, typ: String, list_option_mode: bool) -> ValueRef {
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
                    if list_option_mode {
                        return ValueRef::none();
                    }
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
                    if list_option_mode {
                        return ValueRef::none();
                    }
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
                    if list_option_mode {
                        return ValueRef::none();
                    }
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
                    if list_option_mode {
                        return ValueRef::none();
                    }
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
                    if list_option_mode {
                        return ValueRef::none();
                    }
                    let err_msg = format!("cannot use '{this}' as type '{typ}'");
                    panic!("{}", err_msg);
                }
            }
        }

        if list_option_mode {
            return ValueRef::none();
        }

        panic!("unknown type '{typ}'");
    }

    if let Some(arg0) = args.arg_0() {
        if let Some(x) = ctx.app_args.get(&arg0.as_str()) {
            if *x == 0 {
                return kclvm_value_Undefined();
            }

            let opt_value = mut_ptr_as_ref((*x) as *mut kclvm_value_ref_t);

            if let Some(kwarg_type) = kwargs.kwarg_str("type", None) {
                return _value_to_type(opt_value, kwarg_type, ctx.cfg.list_option_mode).into_raw();
            }

            return (*x) as *mut kclvm_value_ref_t;
        } else if let Some(kwarg_default) = kwargs.kwarg("default") {
            if let Some(kwarg_type) = kwargs.kwarg_str("type", None) {
                return _value_to_type(&kwarg_default, kwarg_type, ctx.cfg.list_option_mode)
                    .into_raw();
            }

            return kwarg_default.into_raw();
        }
    }

    if list_option_mode {
        return kclvm_value_None();
    }

    let required = kwargs.kwarg_bool("required", Some(false)).unwrap();
    if required {
        let name = args.arg_i_str(0, Some("?".to_string())).unwrap();
        panic!("option('{name}') must be initialized, try '-D {name}=?' argument");
    }

    kclvm_value_None()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_print(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    // args
    let list = args.as_list_ref();
    let values: Vec<String> = list.values.iter().map(|v| v.to_string()).collect();
    print!("{}", values.join(" "));
    let dict = kwargs.as_dict_ref();
    // kwargs: end
    if let Some(c) = dict.values.get("end") {
        print!("{c}");
        use std::io::Write;
        let _ = std::io::stdout().flush();
    } else {
        println!();
    }
    kclvm_value_None()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_len(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(arg) = args.arg_0() {
        return kclvm_value_Int(arg.len() as i64);
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
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = args.arg_0() {
        return kclvm_value_Bool(arg0.any_true() as i8);
    }
    kclvm_value_Bool(0)
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_isunique(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = args.arg_0() {
        return kclvm_value_Bool(arg0.isunique() as i8);
    }
    kclvm_value_Bool(0)
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_sorted(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = args.arg_0() {
        let reverse = kwargs.kwarg("reverse");
        return arg0.sorted(reverse.as_ref()).into_raw();
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
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = args.arg_0() {
        let base = args.arg_i(1).or_else(|| kwargs.kwarg("base"));
        return arg0.convert_to_int(base.as_ref()).into_raw();
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
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = args.arg_0() {
        return arg0.convert_to_float().into_raw();
    }
    panic!("float() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_bool(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(arg0) = args.arg_0() {
        return ValueRef::bool(arg0.is_truthy()).into_raw();
    }
    panic!("bool() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(arg0) = args.arg_0() {
        return ValueRef::str(&arg0.to_string()).into_raw();
    }
    ValueRef::str("").into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_max(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);
    if args.args_len() > 1 {
        return args.max_value().into_raw();
    }
    if let Some(arg0) = args.arg_0() {
        return arg0.max_value().into_raw();
    }
    panic!("max() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_min(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);
    if args.args_len() > 1 {
        return args.min_value().into_raw();
    }
    if let Some(arg0) = args.arg_0() {
        return arg0.min_value().into_raw();
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
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);
    if let (Some(arg0), Some(arg1)) = (args.arg_i(0), args.arg_i(1)) {
        return builtin::multiplyof(&arg0, &arg1).into_raw();
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
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);
    if let Some(arg0) = args.arg_0() {
        return arg0.abs().into_raw();
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
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);
    if let Some(arg0) = args.arg_0() {
        return kclvm_value_Bool(arg0.all_true() as i8);
    }
    kclvm_value_Bool(0)
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_hex(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);
    if let Some(arg0) = args.arg_0() {
        return arg0.hex().into_raw();
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
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);
    match args.arg_i(0) {
        Some(arg0) => match args.arg_i(1) {
            Some(arg1) => arg0.sum(&arg1).into_raw(),
            _ => arg0.sum(&ValueRef::int(0)).into_raw(),
        },
        _ => kclvm_value_Undefined(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_pow(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);
    match (args.arg_i(0), args.arg_i(1)) {
        (Some(arg0), Some(arg1)) => match args.arg_i(2) {
            Some(arg2) => builtin::pow(&arg0, &arg1, &arg2).into_raw(),
            _ => builtin::pow(&arg0, &arg1, &ValueRef::none()).into_raw(),
        },
        _ => kclvm_value_Undefined(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_round(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);
    match args.arg_i(0) {
        Some(arg0) => match args.arg_i(1) {
            Some(arg1) => builtin::round(&arg0, &arg1).into_raw(),
            _ => builtin::round(&arg0, &ValueRef::none()).into_raw(),
        },
        _ => kclvm_value_Undefined(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_zip(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);
    args.zip().into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_list(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);
    if args.args_len() > 0 {
        if let Some(arg0) = args.arg_0() {
            return builtin::list(Some(&arg0)).into_raw();
        }
        panic!("invalid arguments in list() function");
    } else {
        builtin::list(None).into_raw()
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_dict(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let mut dict = ValueRef::dict(None);
    if let Some(arg0) = args.arg_0() {
        dict.dict_insert_unpack(&builtin::dict(Some(&arg0)));
    }
    dict.dict_insert_unpack(&builtin::dict(Some(kwargs)));
    dict.into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_typeof(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    if let Some(arg0) = args.arg_0() {
        if let Some(full_name) = kwargs.kwarg("full_name") {
            return builtin::type_of(&arg0, &full_name).into_raw();
        }
        return builtin::type_of(&arg0, &ValueRef::bool(false)).into_raw();
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
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);
    if let Some(arg0) = args.arg_0() {
        return arg0.bin().into_raw();
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
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);
    if let Some(arg0) = args.arg_0() {
        return arg0.oct().into_raw();
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
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);
    if let Some(arg0) = args.arg_0() {
        return arg0.ord().into_raw();
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
    let _ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);
    match args.arg_i(0) {
        Some(arg0) => match args.arg_i(1) {
            Some(arg1) => match args.arg_i(2) {
                Some(arg2) => builtin::range(&arg0, &arg1, &arg2).into_raw(),
                _ => builtin::range(&arg0, &arg1, &ValueRef::int(1)).into_raw(),
            },
            _ => builtin::range(&ValueRef::int(0), &arg0, &ValueRef::int(1)).into_raw(),
        },
        _ => kclvm_value_Undefined(),
    }
}
