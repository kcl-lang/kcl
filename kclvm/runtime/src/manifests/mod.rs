//! KCL manifests system module
//!
//! Copyright 2021 The KCL Authors. All rights reserved.

use crate::*;

#[cfg(test)]
mod tests;
mod yaml;

#[allow(non_camel_case_types)]
type kclvm_value_ref_t = ValueRef;

/// The function is to serialize a list of KCL objects to YAML and output using the style with
/// the `---\n` separator, and put it to the custom manifest output in the context.
///
/// ManifestsYamlStreamOptions contain these options
/// - sort_keys: Sort the encode result by keys (defaults to false).
/// - ignore_private: Whether to ignore the attribute whose name starts with
///     a character `_` (defaults to false).
/// - ignore_none: Whether to ignore the attribute whose value is `None` (defaults to false).
/// - sep: Which separator to use between YAML documents (defaults to "---").
/// More information: https://github.com/kcl-lang/kcl/issues/94
///
/// - Function signature.
///
/// ```kcl, no run
/// schema ManifestsYamlStreamOptions:
///     sort_keys: bool = False
///     ignore_private: bool = True
///     ignore_none: bool = False
///     separator: str = "---\n"
///
/// manifests.yaml_stream(values: [any], * , opts: ManifestsYamlStreamOptions = ManifestsYamlStreamOptions {})
/// ```
///
/// - Usage
///
/// ```kcl, no run
/// import manifests
///
/// config1 = {k1 = "v1"}
/// config2 = {k2 = "v2"}
///
/// manifests.yaml_stream([config1, config2])
/// manifests.yaml_stream([config1, config2], opts = {
///     sort_keys = True
///     ignore_none = True
/// })
/// ```
/// TODO: more options on the function `yaml_stream`.
#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_manifests_yaml_stream(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    // Get the YAML encode options from the second keyword argument `opts`.
    let opts = match kwargs.kwarg("opts").or_else(|| args.arg_i(1)) {
        Some(opts) => {
            if opts.is_config() {
                // Get options or default.
                YamlEncodeOptions {
                    sort_keys: opts
                        .get_by_key("sort_keys")
                        .unwrap_or_else(|| ValueRef::bool(false))
                        .as_bool(),
                    ignore_private: opts
                        .get_by_key("ignore_private")
                        .unwrap_or_else(|| ValueRef::bool(false))
                        .as_bool(),
                    ignore_none: opts
                        .get_by_key("ignore_none")
                        .unwrap_or_else(|| ValueRef::bool(false))
                        .as_bool(),
                    sep: opts
                        .get_by_key("sep")
                        .unwrap_or_else(|| ValueRef::str("---"))
                        .as_str(),
                }
            } else {
                panic!(
                    "Invalid options arguments in yaml_stream(): expect config, got {}",
                    opts.type_str()
                )
            }
        }
        None => YamlEncodeOptions::default(),
    };

    if let Some(value) = args.arg_i(0) {
        self::yaml::encode_yaml_stream_to_manifests(ctx, &value, opts);
    } else {
        panic!("yaml_stream() missing 1 required positional argument: 'values'");
    }
}
