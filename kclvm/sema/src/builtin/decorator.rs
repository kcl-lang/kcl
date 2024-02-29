use std::sync::Arc;

use indexmap::IndexMap;
use once_cell::sync::Lazy;

use crate::ty::{Parameter, Type};

macro_rules! register_decorator {
    ($($name:ident => $ty:expr)*) => (
        // Builtin decorator map.
        pub const BUILTIN_DECORATORS: Lazy<IndexMap<String, Type>> = Lazy::new(|| {
            let mut builtin_mapping = IndexMap::default();
            $( builtin_mapping.insert(stringify!($name).to_string(), $ty); )*
            builtin_mapping
        });
        pub static DECORATOR_NAMES: &[&str] = &[
            $( stringify!($name), )*
        ];
    )
}

register_decorator! {
    deprecated => Type::function(
        None,
        Arc::new(Type::ANY),
        &[
            Parameter {
                name: "version".to_string(),
                ty: Arc::new(Type::STR),
                has_default: true,
            },
            Parameter {
                name: "reason".to_string(),
                ty: Arc::new(Type::STR),
                has_default: true,
            },
            Parameter {
                name: "strict".to_string(),
                ty: Arc::new(Type::BOOL),
                has_default: true,
            },
        ],
        r#"This decorator is used to get the deprecation message according to the wrapped key-value pair."#,
        false,
        None,
    )
    info => Type::function(
        None,
        Arc::new(Type::ANY),
        &[],
        r#"Info decorator is used to mark some compile-time information for external API queries"#,
        true,
        Some(0),
    )
}
