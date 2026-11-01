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
            // Read the key from the schema config in place: `schema_to_dict`
            // would copy the whole config just to look up one key.
            match p.dict_get_value(key) {
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
            ValueRef::func(
                function as *const () as usize as u64,
                0,
                closure,
                "",
                "",
                false,
            )
        }
        // schema instance
        else if p.is_func() {
            let function = match key {
                "instances" => kcl_schema_instances,
                _ => panic!("schema object attr '{key}' not found"),
            };
            let closure = ValueRef::list(Some(&[p]));
            ValueRef::func(
                function as *const () as usize as u64,
                0,
                closure,
                "",
                "",
                false,
            )
        } else {
            panic!(
                "invalid value '{}' to load attribute '{}'",
                p.type_str(),
                key
            );
        }
    }
}

#[cfg(test)]
mod test_value_attr {
    use std::rc::Rc;

    use crate::*;

    const TEST_SCHEMA_NAME: &str = "Data";

    fn get_test_schema_value() -> ValueRef {
        let mut config = ValueRef::dict(None);
        config.dict_update_key_value("int", ValueRef::int(1));
        config.dict_update_key_value("none", ValueRef::none());
        config.dict_update_key_value("undefined", ValueRef::undefined());
        config.dict_update_key_value("dict", ValueRef::dict_str(&[("key", "value")]));
        config.dict_to_schema(
            TEST_SCHEMA_NAME,
            MAIN_PKG_PATH,
            &[],
            &ValueRef::dict(None),
            &ValueRef::dict(None),
            None,
            None,
        )
    }

    #[test]
    fn test_load_attr_dict() {
        let dict = ValueRef::dict_int(&[("key", 1)]);
        assert_eq!(dict.load_attr("key"), ValueRef::int(1));
        assert!(dict.load_attr("missing").is_undefined());
    }

    #[test]
    fn test_load_attr_schema() {
        let schema = get_test_schema_value();
        assert_eq!(schema.load_attr("int"), ValueRef::int(1));
        assert!(schema.load_attr("none").is_none());
        assert!(schema.load_attr("undefined").is_undefined());
        assert_eq!(
            schema.load_attr("dict"),
            ValueRef::dict_str(&[("key", "value")])
        );
        // Reading attributes leaves the schema unchanged.
        assert!(schema.is_schema());
        assert_eq!(schema.len(), 4);
        assert_eq!(schema.type_str(), TEST_SCHEMA_NAME);
    }

    #[test]
    fn test_load_attr_schema_shares_value() {
        let schema = get_test_schema_value();
        let mut dict = schema.load_attr("dict");
        assert!(Rc::ptr_eq(
            &dict.rc,
            &schema.dict_get_value("dict").unwrap().rc
        ));
        // The loaded attribute is the schema's own value, so an update through
        // it is visible through the schema.
        dict.dict_update_key_value("other", ValueRef::str("value"));
        assert_eq!(schema.load_attr("dict").len(), 2);
    }

    #[test]
    fn test_load_attr_schema_reads_current_config() {
        let mut schema = get_test_schema_value();
        let original = schema.load_attr("int");
        assert_eq!(original, ValueRef::int(1));
        schema.dict_update_key_value("int", ValueRef::int(2));
        schema.dict_update_key_value("new", ValueRef::str("value"));
        assert_eq!(original, ValueRef::int(1));
        assert_eq!(schema.load_attr("int"), ValueRef::int(2));
        assert_eq!(schema.load_attr("new"), ValueRef::str("value"));
    }

    #[test]
    #[should_panic(expected = "schema 'Data' attribute 'missing' not found")]
    fn test_load_attr_schema_not_found() {
        get_test_schema_value().load_attr("missing");
    }
}
