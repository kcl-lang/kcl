//! Copyright The KCL Authors. All rights reserved.

use std::sync::Arc;

use crate::ty::{Parameter, Type, TypeRef};
use kclvm_error::diagnostic::dummy_range;
use kclvm_primitives::IndexMap;
use once_cell::sync::Lazy;

// ------------------------------
// base64 system package
// ------------------------------

pub const BASE64: &str = "base64";
macro_rules! register_base64_member {
    ($($name:ident => $ty:expr)*) => (
        pub const BASE64_FUNCTION_TYPES: Lazy<IndexMap<String, Type>> = Lazy::new(|| {
            let mut builtin_mapping = IndexMap::default();
            $( builtin_mapping.insert(stringify!($name).to_string(), $ty); )*
            builtin_mapping
        });
        pub const BASE64_FUNCTION_NAMES: &[&str] = &[
            $( stringify!($name), )*
        ];
    )
}
register_base64_member! {
    encode => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "value".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "encoding".to_string(),
                ty: Type::str_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Encode the string `value` using the codec registered for encoding."#,
        false,
        None,
    )
    decode => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "value".to_string(),
                ty: Type::str_ref(),
                has_default: false,default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Decode the string `value` using the codec registered for encoding."#,
        false,
        None,
    )
}

// ------------------------------
// base32 system package
// ------------------------------

pub const BASE32: &str = "base32";
macro_rules! register_base32_member {
    ($($name:ident => $ty:expr)*) => (
        pub const BASE32_FUNCTION_TYPES: Lazy<IndexMap<String, Type>> = Lazy::new(|| {
            let mut builtin_mapping = IndexMap::default();
            $( builtin_mapping.insert(stringify!($name).to_string(), $ty); )*
            builtin_mapping
        });
        pub const BASE32_FUNCTION_NAMES: &[&str] = &[
            $( stringify!($name), )*
        ];
    )
}
register_base32_member! {
    encode => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "value".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Encode the string `value` using the base32 codec."#,
        false,
        None,
    )
    decode => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "value".to_string(),
                ty: Type::str_ref(),
                has_default: false,default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Decode the string `value` using the base32 codec."#,
        false,
        None,
    )
}

// ------------------------------
// net system package
// ------------------------------

pub const NET: &str = "net";
macro_rules! register_net_member {
    ($($name:ident => $ty:expr)*) => (
        pub const NET_FUNCTION_TYPES: Lazy<IndexMap<String, Type>> = Lazy::new(|| {
            let mut builtin_mapping = IndexMap::default();
            $( builtin_mapping.insert(stringify!($name).to_string(), $ty); )*
            builtin_mapping
        });
        pub const NET_FUNCTION_NAMES: &[&str] = &[
            $( stringify!($name), )*
        ];
    )
}
register_net_member! {
    CIDR_host => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "cidr".to_string(),
                ty: Type::str_ref(),
                has_default: false,default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "host_num".to_string(),
                ty: Type::int_ref(),
                has_default: false,default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Calulate a host IP within an enclosing subnet."#,
        false,
        None,
    )
    CIDR_netmask => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "cidr".to_string(),
                ty: Type::str_ref(),
                has_default: false,default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Calulate the netmask for a subnet."#,
        false,
        None,
    )
    CIDR_subnet => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "cidr".to_string(),
                ty: Type::str_ref(),
                has_default: false,default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "additional_bits".to_string(),
                ty: Type::int_ref(),
                has_default: false,default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "net_num".to_string(),
                ty: Type::int_ref(),
                has_default: false,default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Calulate a subnet within an enclosing subnet."#,
        false,
        None,
    )
    CIDR_subnets => Type::function(
        None,
        Type::list_ref(Type::str_ref()),
        &[
            Parameter {
                name: "cidr".to_string(),
                ty: Type::str_ref(),
                has_default: false,default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "additional_bits".to_string(),
                ty: Type::list_ref(Type::int_ref()),
                has_default: false,default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Allocate subnets within an enclosing subnet."#,
        false,
        None,
    )
    split_host_port => Type::function(
        None,
        Type::list_ref(Type::str_ref()),
        &[
            Parameter {
                name: "ip_end_point".to_string(),
                ty: Type::str_ref(),
                has_default: false,default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Split the `host` and `port` from the `ip_end_point`."#,
        false,
        None,
    )
    join_host_port => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "host".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "port".to_string(),
                ty: Type::union_ref(&[Type::int_ref(), Type::str_ref()]),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Merge the `host` and `port`."#,
        false,
        None,
    )
    fqdn => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "name".to_string(),
                ty: Type::str_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return Fully Qualified Domain Name (FQDN)."#,
        false,
        None,
    )
    parse_IP => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "ip".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Parse ip to a real IP address."#,
        false,
        None,
    )
    IP_string => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "ip".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Get the IP string."#,
        false,
        None,
    )
    to_IP4 => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "ip".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Get the IP4 form of ip."#,
        false,
        None,
    )
    to_IP16 => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "ip".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Get the IP16 form of ip."#,
        false,
        None,
    )
    is_IPv4 => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "ip".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Whether ip is a IPv4 one."#,
        false,
        None,
    )
    is_IP => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "ip".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Whether ip is a valid ip address."#,
        false,
        None,
    )
    is_loopback_IP => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "ip".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Whether ip is a loopback one."#,
        false,
        None,
    )
    is_multicast_IP => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "ip".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Whether ip is a multicast one."#,
        false,
        None,
    )
    is_interface_local_multicast_IP => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "ip".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Whether ip is a interface, local and multicast one."#,
        false,
        None,
    )
    is_link_local_multicast_IP => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "ip".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Whether ip is a link local and multicast one."#,
        false,
        None,
    )
    is_link_local_unicast_IP => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "ip".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Whether ip is a link local and unicast one."#,
        false,
        None,
    )
    is_global_unicast_IP => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "ip".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Whether ip is a global and unicast one."#,
        false,
        None,
    )
    is_unspecified_IP => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "ip".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Whether ip is a unspecified one."#,
        false,
        None,
    )
    parse_CIDR => Type::function(
        None,
        Type::dict_ref(Type::str_ref(), Type::any_ref()),
        &[
            Parameter {
                name: "cidr".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Parse a CIDR prefix into a dict containing 'ip' (the IP) and 'mask' (the prefix bit length)."#,
        false,
        None,
    )
    is_IP_in_CIDR => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "ip".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "cidr".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Check if an IP address is within a given CIDR block."#,
        false,
        None,
    )
}

// ------------------------------
// manifests system package
// ------------------------------

pub const MANIFESTS: &str = "manifests";
macro_rules! register_manifests_member {
    ($($name:ident => $ty:expr)*) => (
        pub const MANIFESTS_FUNCTION_TYPES: Lazy<IndexMap<String, Type>> = Lazy::new(|| {
            let mut builtin_mapping = IndexMap::default();
            $( builtin_mapping.insert(stringify!($name).to_string(), $ty); )*
            builtin_mapping
        });
        pub const MANIFESTS_FUNCTION_NAMES: &[&str] = &[
            $( stringify!($name), )*
        ];
    )
}
register_manifests_member! {
    yaml_stream => Type::function(
        None,
        Type::any_ref(),
        &[
            Parameter {
                name: "values".to_string(),
                ty: Type::any_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "opts".to_string(),
                ty: Type::dict_ref(Type::str_ref(), Type::any_ref()),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"This function is used to serialize the KCL object list into YAML output with the --- separator. It has two parameters:
values - A list of KCL objects
opts - The YAML serialization options
 + sort_keys: Whether to sort the serialized results in the dictionary order of attribute names (the default is False).
 + ignore_private: Whether to ignore the attribute output whose name starts with the character _ (the default value is True).
 + ignore_none: Whether to ignore the attribute with the value of' None '(the default value is False).
 + sep: Set the separator between multiple YAML documents (the default value is "---").
"#,
        false,
        None,
    )
}

// ------------------------------
// math system package
// ------------------------------

pub const MATH: &str = "math";
macro_rules! register_math_member {
    ($($name:ident => $ty:expr)*) => (
        pub const MATH_FUNCTION_TYPES: Lazy<IndexMap<String, Type>> = Lazy::new(|| {
            let mut builtin_mapping = IndexMap::default();
            $( builtin_mapping.insert(stringify!($name).to_string(), $ty); )*
            builtin_mapping
        });
        pub const MATH_FUNCTION_NAMES: &[&str] = &[
            $( stringify!($name), )*
        ];
    )
}
register_math_member! {
    ceil => Type::function(
        None,
        Type::int_ref(),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return the ceiling of `x` as an Integral. This is the smallest integer >= `x`."#,
        false,
        None,
    )
    factorial => Type::function(
        None,
        Type::int_ref(),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return `x`!. Raise a error if `x` is negative or non-integral."#,
        false,
        None,
    )
    floor => Type::function(
        None,
        Type::int_ref(),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return the floor of `x` as an Integral. This is the largest integer <= `x`."#,
        false,
        None,
    )
    gcd => Type::function(
        None,
        Type::int_ref(),
        &[
            Parameter {
                name: "a".to_string(),
                ty: Type::int_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "b".to_string(),
                ty: Type::int_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return the greatest common divisor of `x` and `y`."#,
        false,
        None,
    )
    isfinite => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return `True` if `x` is neither an infinity nor a NaN, and `False` otherwise."#,
        false,
        None,
    )
    isinf => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return `True` if `x` is a positive or negative infinity, and `False` otherwise."#,
        false,
        None,
    )
    isnan => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return `True` if `x` is a NaN (not a number), and `False` otherwise."#,
        false,
        None,
    )
    modf => Type::function(
        None,
        Type::list_ref(Type::float_ref()),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return the fractional and integer parts of `x`. Both results carry the sign of `x` and are floats."#,
        false,
        None,
    )
    exp => Type::function(
        None,
        Type::float_ref(),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return `e` raised to the power of `x`."#,
        false,
        None,
    )
    expm1 => Type::function(
        None,
        Type::float_ref(),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return `exp(x) - 1`. This function avoids the loss of precision involved in the direct evaluation of `exp(x) - 1` for small `x`."#,
        false,
        None,
    )
    log => Type::function(
        None,
        Type::float_ref(),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "e".to_string(),
                ty: Type::float_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return the logarithm of `x` to the base `e`."#,
        false,
        None,
    )
    log1p => Type::function(
        None,
        Type::float_ref(),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return the natural logarithm of `1+x` (base `e`). The result is computed in a way which is accurate for `x` near zero."#,
        false,
        None,
    )
    log2 => Type::function(
        None,
        Type::float_ref(),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return the base 2 logarithm of x."#,
        false,
        None,
    )
    log10 => Type::function(
        None,
        Type::float_ref(),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return the base 10 logarithm of `x`."#,
        false,
        None,
    )
    pow => Type::function(
        None,
        Type::float_ref(),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "y".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return `x**y` (`x` to the power of `y`)."#,
        false,
        None,
    )
    sqrt => Type::function(
        None,
        Type::float_ref(),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return the square root of `x`."#,
        false,
        None,
    )
}

// ------------------------------
// datetime system package
// ------------------------------

pub const DATETIME: &str = "datetime";
macro_rules! register_datetime_member {
    ($($name:ident => $ty:expr)*) => (
        pub const DATETIME_FUNCTION_TYPES: Lazy<IndexMap<String, Type>> = Lazy::new(|| {
            let mut builtin_mapping = IndexMap::default();
            $( builtin_mapping.insert(stringify!($name).to_string(), $ty); )*
            builtin_mapping
        });
        pub const DATETIME_FUNCTION_NAMES: &[&str] = &[
            $( stringify!($name), )*
        ];
    )
}
register_datetime_member! {
    ticks => Type::function(
        None,
        Type::float_ref(),
        &[],
        r#"Return the current time in seconds since the Epoch. Fractions of a second may be present if the system clock provides them."#,
        false,
        None,
    )
    date => Type::function(
        None,
        Type::str_ref(),
        &[],
        r#"Return the `%Y-%m-%d %H:%M:%S` format date."#,
        false,
        None,
    )
    now => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "format".to_string(),
                ty: Type::str_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return the local time format. e.g. 'Sat Jun 06 16:26:11 1998' or format the combined date and time per the specified format string, and the default date format is "%a %b %d %H:%M:%S %Y"."#,
        false,
        None,
    )
    today => Type::function(
        None,
        Type::str_ref(),
        &[],
        r#"Return the `%Y-%m-%d %H:%M:%S.%{ticks}` format date."#,
        false,
        None,
    )
    validate => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "date".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "format".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Validate whether the provided date string matches the specified format."#,
        false,
        None,
    )
}

// ------------------------------
// regex system package
// ------------------------------

pub const REGEX: &str = "regex";
macro_rules! register_regex_member {
    ($($name:ident => $ty:expr)*) => (
        pub const REGEX_FUNCTION_TYPES: Lazy<IndexMap<String, Type>> = Lazy::new(|| {
            let mut builtin_mapping = IndexMap::default();
            $( builtin_mapping.insert(stringify!($name).to_string(), $ty); )*
            builtin_mapping
        });
        pub const REGEX_FUNCTION_NAMES: &[&str] = &[
            $( stringify!($name), )*
        ];
    )
}
register_regex_member! {
    replace => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "string".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "pattern".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "replace".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "count".to_string(),
                ty: Type::int_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return the string obtained by replacing the leftmost non-overlapping occurrences of the pattern in string by the replacement."#,
        false,
        None,
    )
    match => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "string".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "pattern".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Try to apply the pattern at the start of the string, returning a bool value `True` if any match was found, or `False` if no match was found."#,
        false,
        None,
    )
    compile => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "pattern".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Compile a regular expression pattern, returning a bool value denoting whether the pattern is valid."#,
        false,
        None,
    )
    findall => Type::function(
        None,
        Type::list_ref(Type::str_ref()),
        &[
            Parameter {
                name: "string".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "pattern".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return a list of all non-overlapping matches in the string."#,
        false,
        None,
    )
    search => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "string".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "pattern".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Scan through string looking for a match to the pattern, returning a bool value `True` if any match was found, or `False` if no match was found."#,
        false,
        None,
    )
    split => Type::function(
        None,
        Type::list_ref(Type::str_ref()),
        &[
            Parameter {
                name: "string".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "pattern".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "maxsplit".to_string(),
                ty: Type::int_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Return a list composed of words from the string, splitting up to a maximum of `maxsplit` times using `pattern` as the separator."#,
        false,
        None,
    )
}

// ------------------------------
// yaml system package
// ------------------------------

pub const YAML: &str = "yaml";
macro_rules! register_yaml_member {
    ($($name:ident => $ty:expr)*) => (
        pub const YAML_FUNCTION_TYPES: Lazy<IndexMap<String, Type>> = Lazy::new(|| {
            let mut builtin_mapping = IndexMap::default();
            $( builtin_mapping.insert(stringify!($name).to_string(), $ty); )*
            builtin_mapping
        });
        pub const YAML_FUNCTION_NAMES: &[&str] = &[
            $( stringify!($name), )*
        ];
    )
}
register_yaml_member! {
    encode => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "data".to_string(),
                ty: Type::any_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "sort_keys".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "ignore_private".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "ignore_none".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Serialize a KCL object `data` to a YAML formatted str."#,
        false,
        Some(1),
    )
    encode_all => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "data".to_string(),
                ty: Type::list_ref(Type::any_ref()),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "sort_keys".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "ignore_private".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "ignore_none".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Serialize a sequence of KCL objects into a YAML stream str."#,
        false,
        Some(1),
    )
    decode => Type::function(
        None,
        Type::any_ref(),
        &[
            Parameter {
                name: "value".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Deserialize `value` (a string instance containing a YAML document) to a KCL object."#,
        false,
        None,
    )
    decode_all => Type::function(
        None,
        Type::list_ref(Type::any_ref()),
        &[
            Parameter {
                name: "value".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Parse all YAML documents in a stream and produce corresponding KCL objects."#,
        false,
        None,
    )
    dump_to_file => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "data".to_string(),
                ty: Type::any_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "filename".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "sort_keys".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "ignore_private".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "ignore_none".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Serialize a KCL object `data` to a YAML formatted str and write it into the file `filename`."#,
        false,
        Some(2),
    )
    dump_all_to_file => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "data".to_string(),
                ty: Type::list_ref(Type::any_ref()),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "filename".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "sort_keys".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "ignore_private".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "ignore_none".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Serialize a sequence of KCL objects into a YAML stream str and write it into the file `filename`."#,
        false,
        Some(2),
    )
    validate => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "value".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Validate whether the given string is a valid YAML or YAML stream document."#,
        false,
        None,
    )
}

// ------------------------------
// json system package
// ------------------------------

pub const JSON: &str = "json";
macro_rules! register_json_member {
    ($($name:ident => $ty:expr)*) => (
        pub const JSON_FUNCTION_TYPES: Lazy<IndexMap<String, Type>> = Lazy::new(|| {
            let mut builtin_mapping = IndexMap::default();
            $( builtin_mapping.insert(stringify!($name).to_string(), $ty); )*
            builtin_mapping
        });
        pub const JSON_FUNCTION_NAMES: &[&str] = &[
            $( stringify!($name), )*
        ];
    )
}
register_json_member! {
    encode => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "data".to_string(),
                ty: Type::any_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "sort_keys".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "indent".to_string(),
                ty: Type::int_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "ignore_private".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "ignore_none".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Serialize a KCL object `data` to a JSON formatted str."#,
        false,
        Some(1),
    )
    decode => Type::function(
        None,
        Type::any_ref(),
        &[
            Parameter {
                name: "value".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Deserialize `value` (a string instance containing a JSON document) to a KCL object."#,
        false,
        None,
    )
    dump_to_file => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "data".to_string(),
                ty: Type::any_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "filename".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "sort_keys".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "indent".to_string(),
                ty: Type::int_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "ignore_private".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "ignore_none".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Serialize a KCL object `data` to a YAML formatted str and write it into the file `filename`."#,
        false,
        Some(2),
    )
    validate => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "value".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Validate whether the given string is a valid JSON"#,
        false,
        None,
    )
}

// ------------------------------
// crypto system package
// ------------------------------

pub const CRYPTO: &str = "crypto";
macro_rules! register_crypto_member {
    ($($name:ident => $ty:expr)*) => (
        pub const CRYPTO_FUNCTION_TYPES: Lazy<IndexMap<String, Type>> = Lazy::new(|| {
            let mut builtin_mapping = IndexMap::default();
            $( builtin_mapping.insert(stringify!($name).to_string(), $ty); )*
            builtin_mapping
        });
        pub const CRYPTO_FUNCTION_NAMES: &[&str] = &[
            $( stringify!($name), )*
        ];
    )
}
register_crypto_member! {
    md5 => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "value".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "encoding".to_string(),
                ty: Type::str_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Encrypt the string `value` using `MD5` and the codec registered for encoding."#,
        false,
        None,
    )
    sha1 => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "value".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "encoding".to_string(),
                ty: Type::str_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Encrypt the string `value` using `SHA1` and the codec registered for encoding."#,
        false,
        None,
    )
    sha224 => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "value".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "encoding".to_string(),
                ty: Type::str_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Encrypt the string `value` using `SHA224` and the codec registered for encoding."#,
        false,
        None,
    )
    sha256 => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "value".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "encoding".to_string(),
                ty: Type::str_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Encrypt the string `value` using `SHA256` and the codec registered for encoding."#,
        false,
        None,
    )
    sha384 => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "value".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "encoding".to_string(),
                ty: Type::str_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Encrypt the string `value` using `SHA384` and the codec registered for encoding."#,
        false,
        None,
    )
    sha512 => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "value".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "encoding".to_string(),
                ty: Type::str_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Encrypt the string `value` using `SHA512` and the codec registered for encoding."#,
        false,
        None,
    )
    blake3 => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "value".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "encoding".to_string(),
                ty: Type::str_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Encrypt the string `value` using `BLAKE3` and the codec registered for encoding."#,
        false,
        None,
    )
    uuid => Type::function(
        None,
        Type::str_ref(),
        &[],
        r#"Generate a random UUID."#,
        false,
        None,
    )
    filesha256 => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "filepath".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Calculate the SHA256 hash of the file `filepath`."#,
        false,
        None,
    )
}

// ------------------------------
// units system package
// ------------------------------

pub const UNITS: &str = "units";
pub const UNITS_FUNCTION_NAMES: &[&str] = &[
    "to_n", "to_u", "to_m", "to_K", "to_M", "to_G", "to_T", "to_P", "to_Ki", "to_Mi", "to_Gi",
    "to_Ti", "to_Pi",
];
pub const UNITS_NUMBER_MULTIPLIER: &str = "NumberMultiplier";
pub const UNITS_FIELD_NAMES: &[&str] = &[
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
macro_rules! register_units_member {
    ($($name:ident => $ty:expr)*) => (
        pub const UNITS_FUNCTION_TYPES: Lazy<IndexMap<String, Type>> = Lazy::new(|| {
            let mut builtin_mapping = IndexMap::default();
            $( builtin_mapping.insert(stringify!($name).to_string(), $ty); )*
            builtin_mapping
        });
    )
}
register_units_member! {
    n => Type::INT
    u => Type::INT
    m => Type::INT
    k => Type::INT
    K => Type::INT
    M => Type::INT
    G => Type::INT
    T => Type::INT
    P => Type::INT
    Ki => Type::INT
    Mi => Type::INT
    Gi => Type::INT
    Ti => Type::INT
    Pi => Type::INT
    to_n => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "num".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Int literal to string with `n` suffix."#,
        false,
        None,
    )
    to_u => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "num".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Int literal to string with `u` suffix."#,
        false,
        None,
    )
    to_m => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "num".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Int literal to string with `m` suffix."#,
        false,
        None,
    )
    to_K => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "num".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Int literal to string with `K` suffix."#,
        false,
        None,
    )
    to_M => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "num".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Int literal to string with `M` suffix."#,
        false,
        None,
    )
    to_G => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "num".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Int literal to string with `G` suffix."#,
        false,
        None,
    )
    to_T => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "num".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Int literal to string with `T` suffix."#,
        false,
        None,
    )
    to_P => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "num".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Int literal to string with `P` suffix."#,
        false,
        None,
    )
    to_Ki => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "num".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Int literal to string with `Ki` suffix."#,
        false,
        None,
    )
    to_Mi => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "num".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Int literal to string with `Mi` suffix."#,
        false,
        None,
    )
    to_Gi => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "num".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Int literal to string with `Gi` suffix."#,
        false,
        None,
    )
    to_Ti => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "num".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Int literal to string with `Ti` suffix."#,
        false,
        None,
    )
    to_Pi => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "num".to_string(),
                ty: Type::number(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Int literal to string with `Pi` suffix."#,
        false,
        None,
    )
}

// ------------------------------
// collection system package
// ------------------------------

pub const COLLECTION: &str = "collection";
macro_rules! register_collection_member {
    ($($name:ident => $ty:expr)*) => (
        pub const COLLECTION_FUNCTION_TYPES: Lazy<IndexMap<String, Type>> = Lazy::new(|| {
            let mut builtin_mapping = IndexMap::default();
            $( builtin_mapping.insert(stringify!($name).to_string(), $ty); )*
            builtin_mapping
        });
        pub const COLLECTION_FUNCTION_NAMES: &[&str] = &[
            $( stringify!($name), )*
        ];
    )
}
register_collection_member! {
    union_all => Type::function(
        None,
        Type::any_ref(),
        &[
            Parameter {
                name: "num".to_string(),
                ty: Type::list_ref(Type::any_ref()),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Union all object to one object."#,
        false,
        None,
    )
}

// ------------------------------
// file system package
// ------------------------------

pub const FILE: &str = "file";
macro_rules! register_file_member {
    ($($name:ident => $ty:expr)*) => (
        pub const FILE_FUNCTION_TYPES: Lazy<IndexMap<String, Type>> = Lazy::new(|| {
            let mut builtin_mapping = IndexMap::default();
            $( builtin_mapping.insert(stringify!($name).to_string(), $ty); )*
            builtin_mapping
        });
        pub const FILE_FUNCTION_NAMES: &[&str] = &[
            $( stringify!($name), )*
        ];
    )
}
register_file_member! {
    read => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "filepath".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Read the file content from path"#,
        false,
        None,
    )
    glob => Type::function(
        None,
        Type::list_ref(Type::str_ref()),
        &[
            Parameter {
                name: "pattern".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Find all paths that match a pattern"#,
        false,
        None,
    )
    modpath => Type::function(
        None,
        Type::str_ref(),
        &[],
        r#"Read the module root path (kcl.mod file path or a single *.k file path)"#,
        false,
        None,
    )
    workdir => Type::function(
        None,
        Type::str_ref(),
        &[],
        r#"Read the workdir"#,
        false,
        None,
    )
    current => Type::function(
        None,
        Type::str_ref(),
        &[],
        r#"Read the path of the current script or module that is being executed"#,
        false,
        None,
    )
    exists => Type::function(
        None,
        Type::bool_ref(),
        &[
            Parameter {
                name: "filepath".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Whether this file path exists. Returns true if the path points at an existing entity. This function will traverse symbolic links to query information about the destination file."#,
        false,
        None,
    )
    abs => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "filepath".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Returns the canonical, absolute form of the path with all intermediate components normalized and symbolic links resolved."#,
        false,
        None,
    )
    append => Type::function(
        None,
        Type::any_ref(),
        &[
            Parameter {
                name: "filepath".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "content".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Append content to a file at the specified path. If the file doesn't exist, it will be created."#,
        false,
        None,
    )
    mkdir => Type::function(
        None,
        Type::any_ref(),
        &[
            Parameter {
                name: "directory".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "exists".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Create a new directory at the specified path if it doesn't already exist."#,
        false,
        None,
    )
    delete => Type::function(
        None,
        Type::any_ref(),
        &[
            Parameter {
                name: "filepath".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Delete a file or an empty directory at the specified path."#,
        false,
        None,
    )
    cp => Type::function(
        None,
        Type::any_ref(),
        &[
            Parameter {
                name: "src".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "dest".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Copy a file or directory from the source path to the destination path."#,
        false,
        None,
    )
    mv => Type::function(
        None,
        Type::any_ref(),
        &[
            Parameter {
                name: "src".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "dest".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Move a file or directory from the source path to the destination path."#,
        false,
        None,
    )
    size => Type::function(
        None,
        Type::int_ref(),
        &[
            Parameter {
                name: "filepath".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Get the size of a file at the specified path."#,
        false,
        None,
    )
    write => Type::function(
        None,
        Type::any_ref(),
        &[
            Parameter {
                name: "filepath".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "content".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Write content to a file at the specified path. If the file doesn't exist, it will be created. If it does exist, its content will be replaced."#,
        false,
        None,
    )
    read_env => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "key".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Read the environment variable key from the current process."#,
        false,
        None,
    )
}

// ------------------------------
// template system package
// ------------------------------

pub const TEMPLATE: &str = "template";
macro_rules! register_template_member {
    ($($name:ident => $ty:expr)*) => (
        pub const TEMPLATE_FUNCTION_TYPES: Lazy<IndexMap<String, Type>> = Lazy::new(|| {
            let mut builtin_mapping = IndexMap::default();
            $( builtin_mapping.insert(stringify!($name).to_string(), $ty); )*
            builtin_mapping
        });
        pub const TEMPLATE_FUNCTION_NAMES: &[&str] = &[
            $( stringify!($name), )*
        ];
    )
}
register_template_member! {
    execute => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "template".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
            Parameter {
                name: "data".to_string(),
                ty: Type::dict_ref(Type::str_ref(), Type::any_ref()),
                has_default: true,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Applies a parsed template to the specified data object and returns the string output. See https://handlebarsjs.com/ for more documents and examples."#,
        false,
        None,
    )
    html_escape => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "data".to_string(),
                ty: Type::str_ref(),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Replaces the characters `&"<>` with the equivalent html / xml entities."#,
        false,
        None,
    )
}

// ------------------------------
// runtime system package
// ------------------------------

pub const RUNTIME: &str = "runtime";
macro_rules! register_runtime_member {
    ($($name:ident => $ty:expr)*) => (
        pub const RUNTIME_FUNCTION_TYPES: Lazy<IndexMap<String, Type>> = Lazy::new(|| {
            let mut builtin_mapping = IndexMap::default();
            $( builtin_mapping.insert(stringify!($name).to_string(), $ty); )*
            builtin_mapping
        });
        pub const RUNTIME_FUNCTION_NAMES: &[&str] = &[
            $( stringify!($name), )*
        ];
    )
}
register_runtime_member! {
    catch => Type::function(
        None,
        Type::str_ref(),
        &[
            Parameter {
                name: "func".to_string(),
                ty: Arc::new(Type::function(None, Type::any_ref(), &[], "", false, None)),
                has_default: false,
                default_value: None,
                range: dummy_range(),
            },
        ],
        r#"Executes the provided function and catches any potential runtime errors. Returns undefined if execution is successful, otherwise returns an error message in case of a runtime panic."#,
        false,
        None,
    )
}

pub const STANDARD_SYSTEM_MODULES: &[&str] = &[
    COLLECTION, NET, MANIFESTS, MATH, DATETIME, REGEX, YAML, JSON, CRYPTO, BASE64, UNITS, FILE,
    TEMPLATE, RUNTIME, BASE32,
];

pub const STANDARD_SYSTEM_MODULE_NAMES_WITH_AT: &[&str] = &[
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
    "@units",
    "@file",
    "@template",
    "@runtime",
    "@base32",
];

/// Get the system module members
pub fn get_system_module_members(name: &str) -> Vec<&str> {
    match name {
        BASE64 => BASE64_FUNCTION_NAMES.to_vec(),
        BASE32 => BASE32_FUNCTION_NAMES.to_vec(),
        NET => NET_FUNCTION_NAMES.to_vec(),
        MANIFESTS => MANIFESTS_FUNCTION_NAMES.to_vec(),
        MATH => MATH_FUNCTION_NAMES.to_vec(),
        DATETIME => DATETIME_FUNCTION_NAMES.to_vec(),
        REGEX => REGEX_FUNCTION_NAMES.to_vec(),
        YAML => YAML_FUNCTION_NAMES.to_vec(),
        JSON => JSON_FUNCTION_NAMES.to_vec(),
        CRYPTO => CRYPTO_FUNCTION_NAMES.to_vec(),
        UNITS => {
            let mut members = UNITS_FUNCTION_NAMES.to_vec();
            members.append(&mut UNITS_FIELD_NAMES.to_vec());
            members
        }
        COLLECTION => COLLECTION_FUNCTION_NAMES.to_vec(),
        FILE => FILE_FUNCTION_NAMES.to_vec(),
        TEMPLATE => TEMPLATE_FUNCTION_NAMES.to_vec(),
        RUNTIME => RUNTIME_FUNCTION_NAMES.to_vec(),
        _ => bug!("invalid system module name '{}'", name),
    }
}

/// Get the system package member function type, if not found, return the any type.
pub fn get_system_member_function_ty(name: &str, func: &str) -> TypeRef {
    let optional_ty = match name {
        BASE64 => {
            let types = BASE64_FUNCTION_TYPES;
            types.get(func).cloned()
        }
        BASE32 => {
            let types = BASE32_FUNCTION_TYPES;
            types.get(func).cloned()
        }
        NET => {
            let types = NET_FUNCTION_TYPES;
            types.get(func).cloned()
        }
        MANIFESTS => {
            let types = MANIFESTS_FUNCTION_TYPES;
            types.get(func).cloned()
        }
        MATH => {
            let types = MATH_FUNCTION_TYPES;
            types.get(func).cloned()
        }
        DATETIME => {
            let types = DATETIME_FUNCTION_TYPES;
            types.get(func).cloned()
        }
        REGEX => {
            let types = REGEX_FUNCTION_TYPES;
            types.get(func).cloned()
        }
        YAML => {
            let types = YAML_FUNCTION_TYPES;
            types.get(func).cloned()
        }
        JSON => {
            let types = JSON_FUNCTION_TYPES;
            types.get(func).cloned()
        }
        CRYPTO => {
            let types = CRYPTO_FUNCTION_TYPES;
            types.get(func).cloned()
        }
        UNITS => {
            let types = UNITS_FUNCTION_TYPES;
            types.get(func).cloned()
        }
        COLLECTION => {
            let types = COLLECTION_FUNCTION_TYPES;
            types.get(func).cloned()
        }
        FILE => {
            let types = FILE_FUNCTION_TYPES;
            types.get(func).cloned()
        }
        TEMPLATE => {
            let types = TEMPLATE_FUNCTION_TYPES;
            types.get(func).cloned()
        }
        RUNTIME => {
            let types = RUNTIME_FUNCTION_TYPES;
            types.get(func).cloned()
        }
        _ => None,
    };
    optional_ty
        .map(|ty| Arc::new(ty))
        .unwrap_or(Type::any_ref())
}
