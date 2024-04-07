//! This package mainly contains the type definitions of built-in system libraries,
//! functions, decorators and member methods.
pub mod decorator;
pub mod option;
pub mod string;
pub mod system_module;

use std::sync::Arc;

use indexmap::IndexMap;
use once_cell::sync::Lazy;

use crate::ty::{Parameter, Type};
pub use decorator::BUILTIN_DECORATORS;
pub use string::STRING_MEMBER_FUNCTIONS;
pub use system_module::*;

pub const KCL_BUILTIN_FUNCTION_MANGLE_PREFIX: &str = "kclvm_builtin";
pub const KCL_SYSTEM_MODULE_MANGLE_PREFIX: &str = "kclvm_";
pub const BUILTIN_FUNCTION_PREFIX: &str = "$builtin";

macro_rules! register_builtin {
    ($($name:ident => $ty:expr)*) => (
        // Builtin function map.
        pub const BUILTIN_FUNCTIONS: Lazy<IndexMap<String, Type>> = Lazy::new(|| {
            let mut builtin_mapping = IndexMap::default();
            $( builtin_mapping.insert(stringify!($name).to_string(), $ty); )*
            builtin_mapping
        });
        pub static BUILTIN_FUNCTION_NAMES: &[&str] = &[
            $( stringify!($name), )*
        ];
    )
}

register_builtin! {
    option => Type::function(
        None,
        Arc::new(Type::ANY),
        &[
            Parameter {
                name: "key".to_string(),
                ty: Arc::new(Type::STR),
                has_default: false,
            },
            Parameter {
                name: "type".to_string(),
                ty: Arc::new(Type::STR),
                has_default: true,
            },
            Parameter {
                name: "required".to_string(),
                ty: Arc::new(Type::BOOL),
                has_default: true,
            },
            Parameter {
                name: "default".to_string(),
                ty: Arc::new(Type::ANY),
                has_default: true,
            },
            Parameter {
                name: "help".to_string(),
                ty: Arc::new(Type::STR),
                has_default: true,
            },
        ],
        "Return the top level argument by the key",
        false,
        Some(1),
    )
    print => Type::function(
        None,
        Arc::new(Type::NONE),
        &[],
        r#"Prints the values to a stream, or to the system stdout by default.
Optional keyword arguments:
sep:   string inserted between values, default a space.
end:   string appended after the last value, default a newline."#,
        true,
        None,
    )
    multiplyof => Type::function(
        None,
        Arc::new(Type::BOOL),
        &[
            Parameter {
                name: "a".to_string(),
                ty: Arc::new(Type::INT),
                has_default: false,
            },
            Parameter {
                name: "b".to_string(),
                ty: Arc::new(Type::INT),
                has_default: false,
            },
        ],
        "Check if the modular result of a and b is 0.",
        false,
        None,
    )
    isunique => Type::function(
        None,
        Arc::new(Type::BOOL),
        &[
            Parameter {
                name: "inval".to_string(),
                ty: Type::list_ref(Arc::new(Type::ANY)),
                has_default: false,
            },
        ],
        "Check if a list has duplicated elements",
        false,
        None,
    )
    len => Type::function(
        None,
        Arc::new(Type::INT),
        &[
            Parameter {
                name: "inval".to_string(),
                ty: Type::iterable(),
                has_default: false,
            },
        ],
        "Return the length of a value.",
        false,
        None,
    )
    abs => Type::function(
        None,
        Arc::new(Type::ANY),
        &[
            Parameter {
                name: "inval".to_string(),
                ty: Arc::new(Type::ANY),
                has_default: false,
            },
        ],
        "Return the absolute value of the argument.",
        false,
        None,
    )
    all_true => Type::function(
        None,
        Arc::new(Type::BOOL),
        &[
            Parameter {
                name: "inval".to_string(),
                ty: Type::list_ref(Arc::new(Type::ANY)),
                has_default: false,
            },
        ],
        r#"Return True if bool(x) is True for all values x in the iterable.

If the iterable is empty, return True."#,
        false,
        None,
    )
    any_true => Type::function(
        None,
        Arc::new(Type::BOOL),
        &[
            Parameter {
                name: "inval".to_string(),
                ty: Type::list_ref(Arc::new(Type::ANY)),
                has_default: false,
            },
        ],
        r#"Return True if bool(x) is True for any x in the iterable.

If the iterable is empty, return False."#,
        false,
        None,
    )
    hex => Type::function(
        None,
        Arc::new(Type::STR),
        &[
            Parameter {
                name: "number".to_string(),
                ty: Arc::new(Type::INT),
                has_default: false,
            },
        ],
        "Return the hexadecimal representation of an integer.",
        false,
        None,
    )
    bin => Type::function(
        None,
        Arc::new(Type::STR),
        &[
            Parameter {
                name: "number".to_string(),
                ty: Arc::new(Type::INT),
                has_default: false,
            },
        ],
        "Return the binary representation of an integer.",
        false,
        None,
    )
    oct => Type::function(
        None,
        Arc::new(Type::STR),
        &[
            Parameter {
                name: "number".to_string(),
                ty: Arc::new(Type::INT),
                has_default: false,
            },
        ],
        "Return the octal representation of an integer.",
        false,
        None,
    )
    ord => Type::function(
        None,
        Arc::new(Type::INT),
        &[
            Parameter {
                name: "c".to_string(),
                ty: Arc::new(Type::STR),
                has_default: false,
            },
        ],
        "Return the Unicode code point for a one-character string.",
        false,
        None,
    )
    sorted => Type::function(
        None,
        Type::list_ref(Arc::new(Type::ANY)),
        &[
            Parameter {
                name: "inval".to_string(),
                ty: Type::iterable(),
                has_default: false,
            },
            Parameter {
                name: "reverse".to_string(),
                ty: Arc::new(Type::BOOL),
                has_default: true,
            },
        ],
        r#"Return a new list containing all items from the iterable in ascending order.

A custom key function can be supplied to customize the sort order, and the reverse
flag can be set to request the result in descending order."#,
        false,
        Some(1),
    )
    range => Type::function(
        None,
        Type::list_ref(Arc::new(Type::INT)),
        &[
            Parameter {
                name: "start".to_string(),
                ty: Arc::new(Type::INT),
                has_default: true,
            },
            Parameter {
                name: "stop".to_string(),
                ty: Arc::new(Type::INT),
                has_default: true,
            },
            Parameter {
                name: "step".to_string(),
                ty: Arc::new(Type::INT),
                has_default: true,
            },
        ],
        r#"Return the range of a value."#,
        false,
        None,
    )
    max => Type::function(
        None,
        Arc::new(Type::ANY),
        &[],
        r#"With a single iterable argument, return its biggest item.
The default keyword-only argument specifies an object to return
if the provided iterable is empty. With two or more arguments,
return the largest argument."#,
        true,
        None,
    )
    min => Type::function(
        None,
        Arc::new(Type::ANY),
        &[],
        r#"With a single iterable argument, return its smallest item.
The default keyword-only argument specifies an object to return
if the provided iterable is empty. With two or more arguments,
return the smallest argument."#,
        true,
        None,
    )
    sum => Type::function(
        None,
        Arc::new(Type::ANY),
        &[
            Parameter {
                name: "iterable".to_string(),
                ty: Type::list_ref(Arc::new(Type::ANY)),
                has_default: false,
            },
            Parameter {
                name: "start".to_string(),
                ty: Arc::new(Type::ANY),
                has_default: true,
            },
        ],
        r#"When the iterable is empty, return the start value. This function is
intended specifically for use with numeric values and may reject
non-numeric types."#,
        false,
        None,
    )
    pow => Type::function(
        None,
        Type::number(),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Type::number(),
                has_default: false,
            },
            Parameter {
                name: "y".to_string(),
                ty: Type::number(),
                has_default: false,
            },
            Parameter {
                name: "z".to_string(),
                ty: Type::number(),
                has_default: true,
            },
        ],
        r#"Equivalent to `x ** y` (with two arguments) or `x ** y % z` (with three arguments)

Some types, such as ints, are able to use a more efficient algorithm when
invoked using the three argument form."#,
        false,
        None,
    )
    round => Type::function(
        None,
        Type::number(),
        &[
            Parameter {
                name: "number".to_string(),
                ty: Type::number(),
                has_default: false,
            },
            Parameter {
                name: "ndigits".to_string(),
                ty: Arc::new(Type::INT),
                has_default: true,
            },
        ],
        r#"Round a number to a given precision in decimal digits.

The return value is an integer if ndigits is omitted or None.
Otherwise the return value has the same type as the number.
ndigits may be negative."#,
        false,
        None,
    )
    zip => Type::function(
        None,
        Type::list_ref(Arc::new(Type::ANY)),
        &[],
        r#"Return a zip object whose next method returns
a tuple where the i-th element comes from the i-th iterable
argument."#,
        true,
        None,
    )
    int => Type::function(
        None,
        Arc::new(Type::INT),
        &[
            Parameter {
                name: "number".to_string(),
                ty: Arc::new(Type::ANY),
                has_default: false,
            },
            Parameter {
                name: "base".to_string(),
                ty: Arc::new(Type::INT),
                has_default: true,
            },
        ],
        r#"Convert a number or string to an integer, or return 0 if no arguments
are given. For floating point numbers, this truncates towards zero."#,
        false,
        None,
    )
    float => Type::function(
        None,
        Arc::new(Type::FLOAT),
        &[
            Parameter {
                name: "number".to_string(),
                ty: Arc::new(Type::ANY),
                has_default: false,
            },
        ],
        r#"Convert a string or number to a floating point number, if possible."#,
        false,
        None,
    )
    bool => Type::function(
        None,
        Arc::new(Type::BOOL),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Arc::new(Type::ANY),
                has_default: true,
            },
        ],
        r#"Returns True when the argument x is true, False otherwise.
The builtin `True` and `False` are the only two instances of the class bool.
The class bool is a subclass of the class int, and cannot be subclassed."#,
        false,
        None,
    )
    str => Type::function(
        None,
        Arc::new(Type::STR),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Arc::new(Type::ANY),
                has_default: true,
            },
        ],
        r#"Create a new string object from the given object.
If encoding or errors is specified, then the object must
expose a data buffer that will be decoded using the
given encoding and error handler."#,
        false,
        None,
    )
    list => Type::function(
        None,
        Type::list_ref(Arc::new(Type::ANY)),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Arc::new(Type::ANY),
                has_default: true,
            },
        ],
        r#"Built-in list function, which can convert other data types or construct a list.

If no argument is given, the constructor creates a new empty list.
The argument must be an iterable if specified."#,
        false,
        None,
    )
    dict => Type::function(
        None,
        Type::dict_ref(Arc::new(Type::ANY), Arc::new(Type::ANY)),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Arc::new(Type::ANY),
                has_default: true,
            },
        ],
        r#"Built-in dict function.

If no argument is given, the constructor creates a new empty dict."#,
        true,
        None,
    )
    typeof => Type::function(
        None,
        Arc::new(Type::STR),
        &[
            Parameter {
                name: "x".to_string(),
                ty: Arc::new(Type::ANY),
                has_default: false,
            },
            Parameter {
                name: "full_name".to_string(),
                ty: Arc::new(Type::BOOL),
                has_default: true,
            },
        ],
        r#"Return the type of the object"#,
        false,
        None,
    )
}
