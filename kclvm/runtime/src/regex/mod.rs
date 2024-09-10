//! Copyright The KCL Authors. All rights reserved.

extern crate fancy_regex;

use crate::*;

// match(string: str, pattern: str) -> bool:

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_regex_match(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(string) = get_call_arg_str(args, kwargs, 0, Some("string")) {
        if let Some(pattern) = get_call_arg_str(args, kwargs, 1, Some("pattern")) {
            let re = fancy_regex::Regex::new(pattern.as_ref()).unwrap();
            match re.is_match(string.as_ref()) {
                Ok(ok) => {
                    if ok {
                        return kclvm_value_Bool(ctx, 1);
                    } else {
                        return kclvm_value_Bool(ctx, 0);
                    }
                }
                _ => return kclvm_value_Bool(ctx, 0),
            }
        }
    }

    panic!("match() missing 2 required positional arguments: 'string' and 'pattern'")
}

// replace(string: str, pattern: str, replace: str, count: int = 0):

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_regex_replace(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);
    if let Some(string) = get_call_arg_str(args, kwargs, 0, Some("string")) {
        if let Some(pattern) = get_call_arg_str(args, kwargs, 1, Some("pattern")) {
            if let Some(replace) = get_call_arg_str(args, kwargs, 2, Some("replace")) {
                let count = get_call_arg_int(args, kwargs, 3, Some("count")).unwrap_or_else(|| 0);
                let re = fancy_regex::Regex::new(pattern.as_ref()).unwrap();
                let s = re.replacen(string.as_ref(), count as usize, replace.as_ref() as &str);
                return ValueRef::str(&s).into_raw(ctx);
            }
            panic!("replace() missing the required positional argument: 'replace'");
        }
        panic!("replace() missing the required positional argument: 'pattern'");
    }
    panic!("replace() missing 3 required positional arguments: 'string', 'pattern', and 'replace");
}

// compile(pattern: str) -> bool:

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_regex_compile(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(pattern) = get_call_arg_str(args, kwargs, 0, Some("pattern")) {
        match fancy_regex::Regex::new(pattern.as_ref()) {
            Ok(_) => return kclvm_value_Bool(ctx, 1),
            _ => return kclvm_value_Bool(ctx, 0),
        }
    }
    panic!("compile() missing the required positional argument: 'pattern'")
}

// findall(string: str, pattern: str) -> [str]:

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_regex_findall(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);
    if let Some(string) = get_call_arg_str(args, kwargs, 0, Some("string")) {
        if let Some(pattern) = get_call_arg_str(args, kwargs, 1, Some("pattern")) {
            let mut list = ValueRef::list(None);

            for x in fancy_regex::Regex::new(pattern.as_ref())
                .unwrap()
                .captures_iter(string.as_ref())
                .flatten()
            {
                let len = x.len();
                if len < 3 {
                    list.list_append(&ValueRef::str(x.get(0).unwrap().as_str()));
                } else {
                    let mut sub_list = ValueRef::list(None);
                    for i in 1..len {
                        sub_list.list_append(&ValueRef::str(x.get(i).unwrap().as_str()));
                    }
                    list.list_append(&sub_list)
                }
            }

            return list.into_raw(ctx);
        }
        panic!("findall() missing the required positional argument: 'pattern'")
    }
    panic!("findall() missing 2 required positional arguments: 'string' and 'pattern'")
}

// search(string: str, pattern: str):

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_regex_search(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(string) = get_call_arg_str(args, kwargs, 0, Some("string")) {
        if let Some(pattern) = get_call_arg_str(args, kwargs, 1, Some("pattern")) {
            let re = fancy_regex::Regex::new(pattern.as_ref()).unwrap();

            if let Ok(Some(..)) = re.find(string.as_ref()) {
                return kclvm_value_Bool(ctx, 1);
            }
            return kclvm_value_Bool(ctx, 0);
        }
        panic!("search() missing the required positional argument: 'pattern'");
    }
    panic!("search() missing 2 required positional arguments: 'string' and 'pattern'");
}

// split(string: str, pattern: str, maxsplit: int = 0):

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_regex_split(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);
    if let Some(string) = get_call_arg_str(args, kwargs, 0, Some("string")) {
        if let Some(pattern) = get_call_arg_str(args, kwargs, 1, Some("pattern")) {
            let maxsplit = get_call_arg_int(args, kwargs, 2, Some("maxsplit")).unwrap_or_else(|| 0);
            let mut list = ValueRef::list(None);

            let re = fancy_regex::Regex::new(pattern.as_ref()).unwrap();

            let mut fields: Vec<String> = Vec::new();
            let mut current_pos = 0;
            loop {
                let capture = re
                    .captures_from_pos(string.as_ref(), current_pos)
                    .map_or(None, |c| c);
                if let Some(Some(cap)) = capture.map(|c| c.get(0)) {
                    fields.push(string[current_pos..cap.start()].to_string());
                    if maxsplit > 0 && fields.len() >= (maxsplit as usize) {
                        break;
                    }
                    current_pos = cap.end();
                } else {
                    fields.push(string[current_pos..].to_string());
                    break;
                }
            }

            for s in fields {
                list.list_append(&ValueRef::str(s.as_ref()));
            }
            return list.into_raw(ctx);
        }
        panic!("split() missing the required positional argument: 'pattern'");
    }
    panic!("split() missing 2 required positional arguments: 'string' and 'pattern'");
}
