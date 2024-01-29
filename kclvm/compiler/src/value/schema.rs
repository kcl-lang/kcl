//! Copyright The KCL Authors. All rights reserved.

pub const SCHEMA_NAME: &str = "$kclvm_schema";
pub const SCHEMA_ATTR_NAME: &str = "$kclvm_schema_attr";
pub const SCHEMA_CHECK_BLOCK_NAME: &str = "$kclvm_schema_check_block";
pub const SCHEMA_SELF_NAME: &str = "$schema_self";
pub const SCHEMA_CONFIG_NAME: &str = "$schema_config";
pub const SCHEMA_CONFIG_META_NAME: &str = "$schema_config_meta";
pub const SCHEMA_CAL_MAP: &str = "$schema_cal_map";
pub const SCHEMA_ARGS: &str = "$schema_args";
pub const SCHEMA_KWARGS: &str = "$schema_kwargs";
pub const SCHEMA_RUNTIME_TYPE: &str = "$schema_runtime_type";
pub const SCHEMA_VARIABLE_LIST: &[&str] = &[
    BACKTRACK_CACHE,
    BACKTRACK_LEVEL_MAP,
    SCHEMA_CAL_MAP,
    SCHEMA_CONFIG_NAME,
    SCHEMA_CONFIG_META_NAME,
    SCHEMA_SELF_NAME,
    SCHEMA_ARGS,
    SCHEMA_KWARGS,
    SCHEMA_RUNTIME_TYPE,
];
pub const BACKTRACK_LEVEL_MAP: &str = "$backtrack_level_map";
pub const BACKTRACK_CACHE: &str = "$backtrack_cache";

/// KCL schema type
pub struct SchemaType {
    pub name: String,
    pub pkgpath: String,
    pub runtime_type: String,
    pub is_mixin: bool,
    pub is_protocol: bool,
    pub is_rule: bool,
}

impl SchemaType {
    pub fn new(name: &str, pkgpath: &str, runtime_type: &str, is_mixin: bool) -> SchemaType {
        SchemaType {
            name: name.to_string(),
            pkgpath: pkgpath.to_string(),
            runtime_type: runtime_type.to_string(),
            is_mixin,
            is_protocol: false,
            is_rule: false,
        }
    }
}
