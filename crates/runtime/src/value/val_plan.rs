//! Copyright The KCL Authors. All rights reserved.

use crate::*;

pub const KCL_PRIVATE_VAR_PREFIX: &str = "_";
const LIST_DICT_TEMP_KEY: &str = "$";
const SCHEMA_TYPE_META_ATTR: &str = "_type";

/// PlanOptions denotes the configuration required to execute the KCL
/// program and the JSON/YAML planning.
#[derive(PartialEq, Clone, Default, Debug)]
pub struct PlanOptions {
    /// Sorts the key order in the config.
    pub sort_keys: bool,
    /// Emit the `_type` attribute in the schema instance.
    pub include_schema_type_path: bool,
    /// Whether to emit hidden attributes that start with `_`
    pub show_hidden: bool,
    /// Whether to emit none value in the plan process.
    pub disable_none: bool,
    /// Whether to emit empty list in the plan process.
    pub disable_empty_list: bool,
    /// Filter planned value with the path selector.
    pub query_paths: Vec<String>,
    /// YAML plan separator string, default is `---`.
    pub sep: Option<String>,
}

/// Filter list or config results with context options.
fn filter_results(ctx: &Context, key_values: &ValueRef) -> Vec<ValueRef> {
    let mut results: Vec<ValueRef> = vec![];
    // Plan list value with the yaml stream format.
    if key_values.is_list() {
        let key_values_list = &key_values.as_list_ref().values;
        // Check if all elements are configs that need to be expanded for YAML stream format
        let all_configs = key_values_list.iter().all(|v| v.is_config());
        // Check if explicit stream separator is set (e.g., via yaml_stream())
        // If sep is set, always expand to stream format regardless of element types
        let use_stream_format = all_configs || ctx.plan_opts.sep.is_some();

        if use_stream_format {
            // Expand all elements for YAML stream format
            for key_values in key_values_list {
                results.append(&mut filter_results(ctx, key_values));
            }
        } else {
            // Not all configs - preserve the list structure
            // Process nested elements but keep them in a list
            let mut processed_elements: Vec<ValueRef> = vec![];
            for item in key_values_list {
                if item.is_config() {
                    // Config elements might need special handling
                    let filtered = filter_results(ctx, item);
                    if filtered.len() == 1 {
                        processed_elements.push(filtered[0].clone());
                    } else {
                        // Config returned multiple results, append them directly
                        results.append(&mut filtered.clone());
                    }
                } else {
                    // Non-config elements (scalars, lists) - keep as is
                    processed_elements.push(item.clone());
                }
            }
            if !processed_elements.is_empty() {
                results.push(ValueRef::list(Some(
                    &processed_elements.iter().collect::<Vec<_>>(),
                )));
            }
        }
        results
    }
    // Plan dict value
    else if key_values.is_config() {
        let key_values = key_values.as_dict_ref();
        // index 0 for in-line keyvalues output, index 1: for standalone keyvalues outputs
        let mut result = ValueRef::dict(None);
        result.set_potential_schema_type(&key_values.potential_schema.clone().unwrap_or_default());
        results.push(result);
        for (key, value) in &key_values.values {
            if value.is_none() && ctx.plan_opts.disable_none {
                continue;
            }
            if key.starts_with(KCL_PRIVATE_VAR_PREFIX) && !ctx.plan_opts.show_hidden {
                continue;
            }
            if value.is_undefined() || value.is_func() {
                continue;
            } else if value.is_schema() || value.has_potential_schema_type() {
                let filtered = handle_schema(ctx, value);
                if !filtered.is_empty() {
                    // else put it as the value of the key of results
                    let result = results.get_mut(0).unwrap();
                    result.dict_update_key_value(key.as_str(), filtered[0].clone());
                    // if the value has derived 'STANDALONE' instances, extend them
                    if filtered.len() > 1 {
                        for v in &filtered[1..] {
                            results.push(v.clone());
                        }
                    }
                }
            } else if value.is_dict() {
                let filtered = filter_results(ctx, value);
                if !results.is_empty() {
                    let result = results.get_mut(0).unwrap();
                    if !filtered.is_empty() {
                        result.dict_update_key_value(key.as_str(), filtered[0].clone());
                    }
                    // if the value has derived 'STANDALONE' instances, extend them
                    if filtered.len() > 1 {
                        for v in &filtered[1..] {
                            results.push(v.clone());
                        }
                    }
                }
            } else if value.is_list() {
                let mut filtered_list: Vec<ValueRef> = vec![];
                let mut ignore_schema_count = 0;
                let list_value = value.as_list_ref();
                for v in &list_value.values {
                    if v.is_schema() || v.has_potential_schema_type() {
                        let filtered = handle_schema(ctx, v);
                        if filtered.is_empty() {
                            ignore_schema_count += 1;
                            continue;
                        } else {
                            for v in filtered {
                                filtered_list.push(v);
                            }
                        }
                    } else if v.is_dict() {
                        let filtered = filter_results(ctx, v);
                        for v in filtered {
                            filtered_list.push(v);
                        }
                    } else if v.is_none() && ctx.plan_opts.disable_none {
                        continue;
                    } else if !v.is_undefined() {
                        let list_dict = ValueRef::dict(Some(&[(LIST_DICT_TEMP_KEY, v)]));
                        let filtered = filter_results(ctx, &list_dict);
                        if !filtered.is_empty()
                            && let Some(v) = filtered[0].get_by_key(LIST_DICT_TEMP_KEY)
                        {
                            filtered_list.push(v.clone());
                        }
                        if filtered.len() > 1 {
                            for v in &filtered[1..] {
                                results.push(v.clone());
                            }
                        }
                    }
                }
                let schema_in_list_count = ignore_schema_count;
                let value = &value.as_list_ref().values;
                // Plan empty list to values.
                if value.is_empty() && !ctx.plan_opts.disable_empty_list {
                    let result = results.get_mut(0).unwrap();
                    result.dict_update_key_value(key.as_str(), ValueRef::list(None));
                }
                if schema_in_list_count < value.len() {
                    let result = results.get_mut(0).unwrap();
                    let filtered_list: Vec<&ValueRef> = filtered_list.iter().collect();
                    let filtered_list = filtered_list.as_slice();
                    let filtered_list = ValueRef::list(Some(filtered_list));
                    result.dict_update_key_value(key.as_str(), filtered_list);
                }
            } else {
                let result = results.get_mut(0).unwrap();
                result.dict_update_key_value(key.as_str(), value.clone());
            }
        }
        results.iter().enumerate().map(|v| v.1).cloned().collect()
    } else {
        results
    }
}

fn handle_schema(ctx: &Context, value: &ValueRef) -> Vec<ValueRef> {
    let mut filtered = filter_results(ctx, value);
    if filtered.is_empty() {
        return filtered;
    }
    // Deal schema type meta attribute and add the attribute with the type string value
    // into the planned object.
    if ctx.plan_opts.include_schema_type_path
        && let Some(v) = filtered.get_mut(0)
        && v.is_config()
    {
        v.dict_update_key_value(
            SCHEMA_TYPE_META_ATTR,
            ValueRef::str(&value_type_path(value, true)),
        );
    }
    filtered
}

/// Returns the type path of the runtime value `v`.
pub(crate) fn value_type_path(v: &ValueRef, full_name: bool) -> String {
    if v.is_schema() {
        type_of(v, full_name)
    } else {
        match v.get_potential_schema_type() {
            Some(ty_str) => {
                let ty = if full_name {
                    match ty_str.strip_prefix('@') {
                        Some(ty_str) => ty_str.to_string(),
                        None => ty_str.to_string(),
                    }
                } else {
                    let parts: Vec<&str> = ty_str.rsplit('.').collect();
                    match parts.first() {
                        Some(v) => v.to_string(),
                        None => type_of(v, full_name),
                    }
                };
                match ty.strip_prefix(&format!("{MAIN_PKG_PATH}.")) {
                    Some(ty) => ty.to_string(),
                    None => ty,
                }
            }
            None => type_of(v, full_name),
        }
    }
}

/// Returns the type path of the runtime value `v`.
#[inline]
pub fn type_of(v: &ValueRef, full_name: bool) -> String {
    builtin::type_of(v, &ValueRef::bool(full_name)).as_str()
}

impl ValueRef {
    /// Plan the value to JSON and YAML strings.
    pub fn plan(&self, ctx: &Context) -> (String, String) {
        // Encoding options
        let json_opts = JsonEncodeOptions {
            sort_keys: ctx.plan_opts.sort_keys,
            ..Default::default()
        };
        let yaml_opts = YamlEncodeOptions {
            sort_keys: ctx.plan_opts.sort_keys,
            ..Default::default()
        };
        // Filter values with query paths
        let value = if ctx.plan_opts.query_paths.is_empty() {
            self.clone()
        } else {
            self.filter_by_path(&ctx.plan_opts.query_paths)
                .unwrap_or_else(|e| panic!("{e}"))
        };
        if value.is_list_or_config() {
            let results = filter_results(ctx, &value);

            // Plan result using the same approach for both JSON and YAML
            // Serialize the whole filtered value instead of using stream format
            if results.len() == 1 {
                // For config/dict, use the filtered result
                (
                    results[0].to_json_string_with_options(&json_opts),
                    results[0]
                        .to_yaml_string_with_options(&yaml_opts)
                        .strip_suffix('\n')
                        .unwrap()
                        .to_string(),
                )
            } else {
                // Fallback to original value (shouldn't happen normally)
                let sep = ctx
                    .plan_opts
                    .sep
                    .clone()
                    .unwrap_or_else(|| "---".to_string());
                // Plan YAML array results
                let yaml_result = results
                    .iter()
                    .map(|r| {
                        r.to_yaml_string_with_options(&yaml_opts)
                            .strip_suffix('\n')
                            .unwrap()
                            .to_string()
                    })
                    .collect::<Vec<String>>()
                    .join(&format!("\n{}\n", sep));
                // Plan JSON array results
                let json_result = results
                    .iter()
                    .map(|r| r.to_json_string_with_options(&json_opts))
                    .collect::<Vec<String>>()
                    .join(JSON_STREAM_SEP);
                (json_result, yaml_result)
            }
        } else {
            (
                value.to_json_string_with_options(&json_opts),
                value
                    .to_yaml_string_with_options(&yaml_opts)
                    .strip_suffix('\n')
                    .unwrap()
                    .to_string(),
            )
        }
    }

    /// Filter values using path selectors.
    pub fn filter_by_path(&self, path_selector: &[String]) -> Result<ValueRef, String> {
        if self.is_config() && !path_selector.is_empty() {
            if path_selector.len() == 1 {
                let path = &path_selector[0];
                match self.get_by_path(path) {
                    Some(value) => Ok(value),
                    None => Err(format!(
                        "invalid path select operand {path}, value not found"
                    )),
                }
            } else {
                let mut values = ValueRef::list(None);
                for path in path_selector {
                    let value = match self.get_by_path(path) {
                        Some(value) => value,
                        None => {
                            return Err(format!(
                                "invalid path select operand {path}, value not found"
                            ));
                        }
                    };
                    values.list_append(&value);
                }
                Ok(values)
            }
        } else {
            Ok(self.clone())
        }
    }
}

#[cfg(test)]
mod test_value_plan {
    use crate::{Context, MAIN_PKG_PATH, ValueRef, schema_runtime_type, val_plan::PlanOptions};

    use super::filter_results;

    const TEST_SCHEMA_NAME: &str = "Data";

    fn get_test_schema_value() -> ValueRef {
        let mut schema = ValueRef::dict(None).dict_to_schema(
            TEST_SCHEMA_NAME,
            MAIN_PKG_PATH,
            &[],
            &ValueRef::dict(None),
            &ValueRef::dict(None),
            None,
            None,
        );
        schema.set_potential_schema_type(&schema_runtime_type(TEST_SCHEMA_NAME, MAIN_PKG_PATH));
        schema
    }

    fn get_test_schema_value_with_pkg() -> ValueRef {
        let mut schema = ValueRef::dict(None).dict_to_schema(
            TEST_SCHEMA_NAME,
            "pkg",
            &[],
            &ValueRef::dict(None),
            &ValueRef::dict(None),
            None,
            None,
        );
        schema.set_potential_schema_type(&schema_runtime_type(TEST_SCHEMA_NAME, MAIN_PKG_PATH));
        schema
    }

    #[test]
    fn test_filter_results() {
        let ctx = Context::new();
        let dict1 = ValueRef::dict_int(&[("k1", 1)]);
        let dict2 = ValueRef::dict_int(&[("k2", 2)]);
        let dict3 = ValueRef::dict_int(&[("k3", 3)]);
        let dict_list = vec![&dict1, &dict2, &dict3];
        let list_data = ValueRef::list(Some(&dict_list));
        assert_eq!(
            filter_results(&ctx, &list_data),
            dict_list
                .iter()
                .map(|v| v.deep_copy())
                .collect::<Vec<ValueRef>>()
        );
        for dict in dict_list {
            assert_eq!(filter_results(&ctx, dict), vec![dict.deep_copy()]);
        }
    }

    #[test]
    fn test_filter_by_path() {
        let dict = ValueRef::dict_int(&[("k1", 1)]);
        assert_eq!(
            dict.filter_by_path(&[]).unwrap(),
            ValueRef::dict_int(&[("k1", 1)]),
        );
        assert_eq!(
            dict.filter_by_path(&["k1".to_string()]).unwrap(),
            ValueRef::int(1)
        );
        assert_eq!(
            dict.filter_by_path(&["k1".to_string(), "k1".to_string()])
                .unwrap(),
            ValueRef::list_int(&[1, 1])
        );
        assert_eq!(
            dict.filter_by_path(&["err_path".to_string()])
                .err()
                .unwrap(),
            "invalid path select operand err_path, value not found"
        );
        assert_eq!(
            dict.filter_by_path(&["err_path.to".to_string()])
                .err()
                .unwrap(),
            "invalid path select operand err_path.to, value not found"
        );
    }

    #[test]
    fn test_value_plan_with_options() {
        let mut ctx = Context::new();
        ctx.plan_opts = PlanOptions::default();
        let mut config = ValueRef::dict(None);
        config.dict_update_key_value("data", get_test_schema_value());
        config.dict_update_key_value("_hidden", ValueRef::int(1));
        config.dict_update_key_value("vec", ValueRef::list(None));
        config.dict_update_key_value("empty", ValueRef::none());
        let (json_string, yaml_string) = config.plan(&ctx);
        assert_eq!(json_string, "{\"data\": {}, \"vec\": [], \"empty\": null}");
        assert_eq!(yaml_string, "data: {}\nvec: []\nempty: null");

        ctx.plan_opts.include_schema_type_path = true;
        let (json_string, yaml_string) = config.plan(&ctx);
        assert_eq!(
            json_string,
            "{\"data\": {\"_type\": \"Data\"}, \"vec\": [], \"empty\": null}"
        );
        assert_eq!(yaml_string, "data:\n  _type: Data\nvec: []\nempty: null");

        ctx.plan_opts.show_hidden = true;
        let (json_string, yaml_string) = config.plan(&ctx);
        assert_eq!(
            json_string,
            "{\"data\": {\"_type\": \"Data\"}, \"_hidden\": 1, \"vec\": [], \"empty\": null}"
        );
        assert_eq!(
            yaml_string,
            "data:\n  _type: Data\n_hidden: 1\nvec: []\nempty: null"
        );

        ctx.plan_opts.sort_keys = true;
        let (json_string, yaml_string) = config.plan(&ctx);
        assert_eq!(
            json_string,
            "{\"_hidden\": 1, \"data\": {\"_type\": \"Data\"}, \"empty\": null, \"vec\": []}"
        );
        assert_eq!(
            yaml_string,
            "_hidden: 1\ndata:\n  _type: Data\nempty: null\nvec: []"
        );

        ctx.plan_opts.disable_none = true;
        let (json_string, yaml_string) = config.plan(&ctx);
        assert_eq!(
            json_string,
            "{\"_hidden\": 1, \"data\": {\"_type\": \"Data\"}, \"vec\": []}"
        );
        assert_eq!(yaml_string, "_hidden: 1\ndata:\n  _type: Data\nvec: []");

        ctx.plan_opts.disable_empty_list = true;
        let (json_string, yaml_string) = config.plan(&ctx);
        assert_eq!(
            json_string,
            "{\"_hidden\": 1, \"data\": {\"_type\": \"Data\"}}"
        );
        assert_eq!(yaml_string, "_hidden: 1\ndata:\n  _type: Data");

        config.dict_update_key_value("data_with_pkg", get_test_schema_value_with_pkg());
        let (json_string, yaml_string) = config.plan(&ctx);
        assert_eq!(
            json_string,
            "{\"_hidden\": 1, \"data\": {\"_type\": \"Data\"}, \"data_with_pkg\": {\"_type\": \"pkg.Data\"}}"
        );
        assert_eq!(
            yaml_string,
            "_hidden: 1\ndata:\n  _type: Data\ndata_with_pkg:\n  _type: pkg.Data"
        );

        ctx.plan_opts.query_paths = vec!["data".to_string()];
        let (json_string, yaml_string) = config.plan(&ctx);
        assert_eq!(json_string, "{}");
        assert_eq!(yaml_string, "{}");
    }
}
