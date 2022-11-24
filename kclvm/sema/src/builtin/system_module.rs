// Copyright 2021 The KCL Authors. All rights reserved.

pub const BASE64: &str = "base64";
pub const BASE64_FUNCTION_NAMES: [&str; 2] = ["encode", "decode"];

pub const NET: &str = "net";
pub const NET_FUNCTION_NAMES: [&str; 16] = [
    "split_host_port",
    "join_host_port",
    "fqdn",
    "parse_IP",
    "to_IP4",
    "to_IP16",
    "IP_string",
    "is_IPv4",
    "is_IP",
    "is_loopback_IP",
    "is_multicast_IP",
    "is_interface_local_multicast_IP",
    "is_link_local_multicast_IP",
    "is_link_local_unicast_IP",
    "is_global_unicast_IP",
    "is_unspecified_IP",
];

pub const MANIFESTS: &str = "manifests";
pub const MANIFESTS_FUNCTION_NAMES: [&str; 1] = ["yaml_stream"];

pub const MATH: &str = "math";
pub const MATH_FUNCTION_NAMES: [&str; 16] = [
    "ceil",
    "factorial",
    "floor",
    "gcd",
    "isfinite",
    "isinf",
    "isnan",
    "modf",
    "exp",
    "expm1",
    "log",
    "log1p",
    "log2",
    "log10",
    "pow",
    "sqrt",
];

pub const DATETIME: &str = "datetime";
pub const DATETIME_FUNCTION_NAMES: [&str; 4] = ["today", "now", "ticks", "date"];

pub const REGEX: &str = "regex";
pub const REGEX_FUNCTION_NAMES: [&str; 6] =
    ["replace", "match", "compile", "findall", "search", "split"];

pub const YAML: &str = "yaml";
pub const YAML_FUNCTION_NAMES: [&str; 3] = ["encode", "decode", "dump_to_file"];

pub const JSON: &str = "json";
pub const JSON_FUNCTION_NAMES: [&str; 3] = ["encode", "decode", "dump_to_file"];

pub const CRYPTO: &str = "crypto";
pub const CRYPTO_FUNCTION_NAMES: [&str; 6] =
    ["md5", "sha1", "sha224", "sha256", "sha384", "sha512"];

pub const TESTING: &str = "testing";
pub const TESTING_FUNCTION_NAMES: [&str; 2] = ["arguments", "setting_file"];

pub const UNITS: &str = "units";
pub const UNITS_FUNCTION_NAMES: [&str; 13] = [
    "to_n", "to_u", "to_m", "to_K", "to_M", "to_G", "to_T", "to_P", "to_Ki", "to_Mi", "to_Gi",
    "to_Ti", "to_Pi",
];
pub const UNITS_NUMBER_MULTIPLIER: &str = "NumberMultiplier";
pub const UNITS_FIELD_NAMES: [&str; 15] = [
    "n",
    "u",
    "m",
    "k",
    "K",
    "M",
    "G",
    "T",
    "P",
    "Ki",
    "Mi",
    "Gi",
    "Ti",
    "Pi",
    UNITS_NUMBER_MULTIPLIER,
];

pub const COLLECTION: &str = "collection";
pub const COLLECTION_FUNCTION_NAMES: [&str; 1] = ["union_all"];

pub const STANDARD_SYSTEM_MODULES: [&str; 12] = [
    COLLECTION, NET, MANIFESTS, MATH, DATETIME, REGEX, YAML, JSON, CRYPTO, BASE64, TESTING, UNITS,
];

pub const STANDARD_SYSTEM_MODULE_NAMES_WITH_AT: [&str; 12] = [
    "@collection",
    "@net",
    "@manifests",
    "@math",
    "@datetime",
    "@regex",
    "@yaml",
    "@json",
    "@crypto",
    "@base64",
    "@testing",
    "@units",
];

/// Get the system module members
pub fn get_system_module_members(name: &str) -> Vec<&str> {
    match name {
        BASE64 => BASE64_FUNCTION_NAMES.to_vec(),
        NET => NET_FUNCTION_NAMES.to_vec(),
        MANIFESTS => MANIFESTS_FUNCTION_NAMES.to_vec(),
        MATH => MATH_FUNCTION_NAMES.to_vec(),
        DATETIME => DATETIME_FUNCTION_NAMES.to_vec(),
        REGEX => REGEX_FUNCTION_NAMES.to_vec(),
        YAML => YAML_FUNCTION_NAMES.to_vec(),
        JSON => JSON_FUNCTION_NAMES.to_vec(),
        CRYPTO => CRYPTO_FUNCTION_NAMES.to_vec(),
        TESTING => TESTING_FUNCTION_NAMES.to_vec(),
        UNITS => {
            let mut members = UNITS_FUNCTION_NAMES.to_vec();
            members.append(&mut UNITS_FIELD_NAMES.to_vec());
            members
        }
        COLLECTION => COLLECTION_FUNCTION_NAMES.to_vec(),
        _ => bug!("invalid system module name '{}'", name),
    }
}
