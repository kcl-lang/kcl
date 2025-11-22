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
                "lower" => kcl_builtin_str_lower,
                "upper" => kcl_builtin_str_upper,
                "capitalize" => kcl_builtin_str_capitalize,
                "chars" => kcl_builtin_str_chars,
                "count" => kcl_builtin_str_count,
                "endswith" => kcl_builtin_str_endswith,
                "find" => kcl_builtin_str_find,
                "format" => kcl_builtin_str_format,
                "index" => kcl_builtin_str_index,
                "isalnum" => kcl_builtin_str_isalnum,
                "isalpha" => kcl_builtin_str_isalpha,
                "isdigit" => kcl_builtin_str_isdigit,
                "islower" => kcl_builtin_str_islower,
                "isspace" => kcl_builtin_str_isspace,
                "istitle" => kcl_builtin_str_istitle,
                "isupper" => kcl_builtin_str_isupper,
                "join" => kcl_builtin_str_join,
                "lstrip" => kcl_builtin_str_lstrip,
                "rstrip" => kcl_builtin_str_rstrip,
                "replace" => kcl_builtin_str_replace,
                "removeprefix" => kcl_builtin_str_removeprefix,
                "removesuffix" => kcl_builtin_str_removesuffix,
                "rfind" => kcl_builtin_str_rfind,
                "rindex" => kcl_builtin_str_rindex,
                "rsplit" => kcl_builtin_str_rsplit,
                "split" => kcl_builtin_str_split,
                "splitlines" => kcl_builtin_str_splitlines,
                "startswith" => kcl_builtin_str_startswith,
                "strip" => kcl_builtin_str_strip,
                "title" => kcl_builtin_str_title,
                _ => panic!("str object attr '{key}' not found"),
            };
            let closure = ValueRef::list(Some(&[p]));
            ValueRef::func(function as usize as u64, 0, closure, "", "", false)
        }
        // schema instance
        else if p.is_func() {
            let function = match key {
                "instances" => kcl_schema_instances,
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
