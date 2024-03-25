//! Copyright The KCL Authors. All rights reserved.

use crate::*;

impl ValueRef {
    /// Load attribute named `key` from the self value, the attribute maybe a
    /// member function or a builtin value.
    pub fn load_attr(&self, key: &str) -> Self {
        let p: &ValueRef = self;
        // load_attr including str/dict/schema.
        if p.is_dict() {
            match p.dict_get_value(key) {
                Some(x) => x,
                None => ValueRef::undefined(),
            }
        } else if p.is_schema() {
            let dict = p.schema_to_dict();
            match dict.dict_get_value(key) {
                Some(x) => x,
                None => panic!("schema '{}' attribute '{}' not found", p.type_str(), key),
            }
        } else if p.is_str() {
            let function = match key {
                "lower" => kclvm_builtin_str_lower,
                "upper" => kclvm_builtin_str_upper,
                "capitalize" => kclvm_builtin_str_capitalize,
                "count" => kclvm_builtin_str_count,
                "endswith" => kclvm_builtin_str_endswith,
                "find" => kclvm_builtin_str_find,
                "format" => kclvm_builtin_str_format,
                "index" => kclvm_builtin_str_index,
                "isalnum" => kclvm_builtin_str_isalnum,
                "isalpha" => kclvm_builtin_str_isalpha,
                "isdigit" => kclvm_builtin_str_isdigit,
                "islower" => kclvm_builtin_str_islower,
                "isspace" => kclvm_builtin_str_isspace,
                "istitle" => kclvm_builtin_str_istitle,
                "isupper" => kclvm_builtin_str_isupper,
                "join" => kclvm_builtin_str_join,
                "lstrip" => kclvm_builtin_str_lstrip,
                "rstrip" => kclvm_builtin_str_rstrip,
                "replace" => kclvm_builtin_str_replace,
                "removeprefix" => kclvm_builtin_str_removeprefix,
                "removesuffix" => kclvm_builtin_str_removesuffix,
                "rfind" => kclvm_builtin_str_rfind,
                "rindex" => kclvm_builtin_str_rindex,
                "rsplit" => kclvm_builtin_str_rsplit,
                "split" => kclvm_builtin_str_split,
                "splitlines" => kclvm_builtin_str_splitlines,
                "startswith" => kclvm_builtin_str_startswith,
                "strip" => kclvm_builtin_str_strip,
                "title" => kclvm_builtin_str_title,
                _ => panic!("str object attr '{key}' not found"),
            };
            let closure = ValueRef::list(Some(&[p]));
            ValueRef::func(function as usize as u64, 0, closure, "", "", false)
        }
        // schema instance
        else if p.is_func() {
            let function = match key {
                "instances" => kclvm_schema_instances,
                _ => panic!("schema object attr '{key}' not found"),
            };
            let closure = ValueRef::list(Some(&[p]));
            ValueRef::func(function as usize as u64, 0, closure, "", "", false)
        } else {
            panic!(
                "invalid value '{}' to load attribute '{}'",
                p.type_str(),
                key
            );
        }
    }
}
