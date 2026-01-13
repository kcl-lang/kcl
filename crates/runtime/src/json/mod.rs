//! Copyright The KCL Authors. All rights reserved.

use crate::*;

// data, sort_keys=False, indent=None, ignore_private=False, ignore_none=False

/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_json_encode(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("data")) {
        let s = ValueRef::str(
            arg0.to_json_string_with_options(&args_to_opts(args, kwargs, 1))
                .as_ref(),
        );
        return s.into_raw(ctx);
    }
    panic!("encode() missing 1 required positional argument: 'value'")
}

/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_json_decode(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("value")) {
        match ValueRef::from_json(ctx, arg0.as_str().as_ref()) {
            Ok(x) => return x.into_raw(ctx),
            Err(err) => panic!("{}", err),
        }
    }
    panic!("decode() missing 1 required positional argument: 'value'")
}

/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_json_validate(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("value")) {
        match ValueRef::from_json(ctx, arg0.as_str().as_ref()) {
            Ok(_) => return unsafe { kcl_value_True(ctx) },
            Err(_) => return unsafe { kcl_value_False(ctx) },
        }
    }
    panic!("validate() missing 1 required positional argument: 'value'")
}

/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_json_dump_to_file(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    let data = args.arg_i(0).or(kwargs.get_by_key("data"));
    let filename = args.arg_i(1).or(kwargs.get_by_key("filename"));
    match (data, filename) {
        (Some(data), Some(filename)) => {
            let filename = filename.as_str();
            let json = data.to_json_string_with_options(&args_to_opts(args, kwargs, 2));
            std::fs::write(&filename, json)
                .unwrap_or_else(|e| panic!("Unable to write file '{}': {}", filename, e));
            unsafe { kcl_value_Undefined(ctx) }
        }
        _ => {
            panic!("dump_to_file() missing 2 required positional arguments: 'data' and 'filename'")
        }
    }
}

fn args_to_opts(args: &ValueRef, kwargs: &ValueRef, index: usize) -> JsonEncodeOptions {
    let mut opts = JsonEncodeOptions::default();
    if let Some(sort_keys) = get_call_arg_bool(args, kwargs, index, Some("sort_keys")) {
        opts.sort_keys = sort_keys;
    }
    if let Some(indent) = get_call_arg_int(args, kwargs, index + 1, Some("indent")) {
        opts.indent = indent;
    }
    if let Some(ignore_private) = get_call_arg_bool(args, kwargs, index + 2, Some("ignore_private"))
    {
        opts.ignore_private = ignore_private;
    }
    if let Some(ignore_none) = get_call_arg_bool(args, kwargs, index + 3, Some("ignore_none")) {
        opts.ignore_none = ignore_none;
    }
    opts
}

/// merge(src, patch) -> any
/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_json_merge(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };

    let src = get_call_arg(args, kwargs, 0, Some("src"));
    let patch = get_call_arg(args, kwargs, 1, Some("patch"));

    match (src, patch) {
        (Some(src), Some(patch)) => {
            let result = json_merge_impl(&src, &patch);
            result.into_raw(ctx)
        }
        _ => {
            panic!("merge() missing required positional arguments: 'src' and 'patch'")
        }
    }
}

/// RFC 7396 JSON Merge Patch implementation
fn json_merge_impl(src: &ValueRef, patch: &ValueRef) -> ValueRef {
    if !patch.is_config() {
        return patch.deep_copy();
    }

    let mut result = if src.is_config() {
        src.deep_copy()
    } else {
        ValueRef::dict(None)
    };

    let patch_keys = patch.dict_keys();
    let keys = patch_keys.as_list_ref();
    for key_val in keys.values.iter() {
        let key = key_val.as_str();
        if let Some(patch_value) = patch.dict_get_value(&key) {
            if patch_value.is_none_or_undefined() {
                result.dict_remove(&key);
            } else if patch_value.is_config() {
                let src_value = result
                    .dict_get_value(&key)
                    .unwrap_or_else(|| ValueRef::dict(None));
                let merged = json_merge_impl(&src_value, &patch_value);
                result.dict_update_key_value(&key, merged);
            } else {
                result.dict_update_key_value(&key, patch_value.deep_copy());
            }
        }
    }

    result
}

#[cfg(test)]
mod test_json_merge {
    use super::*;

    #[test]
    fn test_basic_merge() {
        // {a: 1, b: 2} + {b: 3, c: 4} = {a: 1, b: 3, c: 4}
        let base = ValueRef::dict_int(&[("a", 1), ("b", 2)]);
        let patch = ValueRef::dict_int(&[("b", 3), ("c", 4)]);
        let result = json_merge_impl(&base, &patch);

        assert_eq!(result.dict_get_value("a").unwrap().as_int(), 1);
        assert_eq!(result.dict_get_value("b").unwrap().as_int(), 3);
        assert_eq!(result.dict_get_value("c").unwrap().as_int(), 4);
    }

    #[test]
    fn test_nested_merge() {
        // {x: {a: 1, b: 2}} + {x: {b: 20, c: 30}} = {x: {a: 1, b: 20, c: 30}}
        let inner_base = ValueRef::dict_int(&[("a", 1), ("b", 2)]);
        let mut base = ValueRef::dict(None);
        base.dict_update_key_value("x", inner_base);

        let inner_patch = ValueRef::dict_int(&[("b", 20), ("c", 30)]);
        let mut patch = ValueRef::dict(None);
        patch.dict_update_key_value("x", inner_patch);

        let result = json_merge_impl(&base, &patch);

        let x = result.dict_get_value("x").unwrap();
        assert_eq!(x.dict_get_value("a").unwrap().as_int(), 1);
        assert_eq!(x.dict_get_value("b").unwrap().as_int(), 20);
        assert_eq!(x.dict_get_value("c").unwrap().as_int(), 30);
    }

    #[test]
    fn test_delete_with_none() {
        // {a: 1, b: 2} + {b: None} = {a: 1}
        let base = ValueRef::dict_int(&[("a", 1), ("b", 2)]);
        let mut patch = ValueRef::dict(None);
        patch.dict_update_key_value("b", ValueRef::none());

        let result = json_merge_impl(&base, &patch);

        assert_eq!(result.dict_get_value("a").unwrap().as_int(), 1);
        assert!(result.dict_get_value("b").is_none());
    }

    #[test]
    fn test_replace_nested_with_scalar() {
        // {a: {nested: "value"}} + {a: "replaced"} = {a: "replaced"}
        let mut nested = ValueRef::dict(None);
        nested.dict_update_key_value("nested", ValueRef::str("value"));
        let mut base = ValueRef::dict(None);
        base.dict_update_key_value("a", nested);

        let mut patch = ValueRef::dict(None);
        patch.dict_update_key_value("a", ValueRef::str("replaced"));

        let result = json_merge_impl(&base, &patch);

        assert_eq!(result.dict_get_value("a").unwrap().as_str(), "replaced");
    }

    #[test]
    fn test_empty_patch() {
        // {a: 1, b: 2} + {} = {a: 1, b: 2}
        let base = ValueRef::dict_int(&[("a", 1), ("b", 2)]);
        let patch = ValueRef::dict(None);
        let result = json_merge_impl(&base, &patch);

        assert_eq!(result.dict_get_value("a").unwrap().as_int(), 1);
        assert_eq!(result.dict_get_value("b").unwrap().as_int(), 2);
    }

    #[test]
    fn test_non_config_replacement() {
        // If patch is not a config, just return patch
        let base = ValueRef::dict_int(&[("a", 1)]);
        let patch = ValueRef::str("replaced");
        let result = json_merge_impl(&base, &patch);

        assert_eq!(result.as_str(), "replaced");
    }
}
