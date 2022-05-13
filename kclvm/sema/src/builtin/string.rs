use indexmap::IndexMap;
use once_cell::sync::Lazy;
use std::rc::Rc;

use crate::ty::Type;

macro_rules! register_string_member {
    ($($name:ident => $ty:expr)*) => (
        // Builtin string member function map.
        pub const STRING_MEMBER_FUNCTIONS: Lazy<IndexMap<String, Type>> = Lazy::new(|| {
            let mut builtin_mapping = IndexMap::default();
            $( builtin_mapping.insert(stringify!($name).to_string(), $ty); )*
            builtin_mapping
        });
    )
}

register_string_member! {
    capitalize => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::ANY),
        &[],
        r#""#,
        false,
        None,
    )
    count => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::INT),
        &[],
        r#""#,
        false,
        None,
    )
    endswith => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::BOOL),
        &[],
        r#""#,
        false,
        None,
    )
    find => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::INT),
        &[],
        r#""#,
        false,
        None,
    )
    format => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::STR),
        &[],
        r#""#,
        true,
        None,
    )
    index => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::INT),
        &[],
        r#""#,
        false,
        None,
    )
    isalpha => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::BOOL),
        &[],
        r#""#,
        false,
        None,
    )
    isalnum => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::BOOL),
        &[],
        r#""#,
        false,
        None,
    )
    isdigit => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::BOOL),
        &[],
        r#""#,
        false,
        None,
    )
    islower => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::BOOL),
        &[],
        r#""#,
        false,
        None,
    )
    isspace => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::BOOL),
        &[],
        r#""#,
        false,
        None,
    )
    istitle => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::BOOL),
        &[],
        r#""#,
        false,
        None,
    )
    isupper => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::BOOL),
        &[],
        r#""#,
        false,
        None,
    )
    join => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::STR),
        &[],
        r#""#,
        true,
        None,
    )
    lower => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::STR),
        &[],
        r#""#,
        true,
        None,
    )
    upper => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::STR),
        &[],
        r#""#,
        true,
        None,
    )
    lstrip => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::STR),
        &[],
        r#""#,
        true,
        None,
    )
    rstrip => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::STR),
        &[],
        r#""#,
        true,
        None,
    )
    replace => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::STR),
        &[],
        r#""#,
        true,
        None,
    )
    rfind => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::INT),
        &[],
        r#""#,
        true,
        None,
    )
    rindex => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::INT),
        &[],
        r#""#,
        true,
        None,
    )
    rsplit => Type::function(
        Some(Rc::new(Type::STR)),
        Type::list_ref(Rc::new(Type::STR)),
        &[],
        r#""#,
        true,
        None,
    )
    split => Type::function(
        Some(Rc::new(Type::STR)),
        Type::list_ref(Rc::new(Type::STR)),
        &[],
        r#""#,
        true,
        None,
    )
    splitlines => Type::function(
        Some(Rc::new(Type::STR)),
        Type::list_ref(Rc::new(Type::STR)),
        &[],
        r#""#,
        true,
        None,
    )
    startswith => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::BOOL),
        &[],
        r#""#,
        false,
        None,
    )
    strip => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::STR),
        &[],
        r#""#,
        false,
        None,
    )
    title => Type::function(
        Some(Rc::new(Type::STR)),
        Rc::new(Type::STR),
        &[],
        r#""#,
        false,
        None,
    )
}
