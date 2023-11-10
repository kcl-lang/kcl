use std::sync::Arc;

use indexmap::IndexMap;
use once_cell::sync::Lazy;

use crate::ty::{Parameter, Type};

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
        Some(Arc::new(Type::STR)),
        Arc::new(Type::STR),
        &[],
        r#"Return a copy of the string with its first character capitalized and the rest lowercased."#,
        false,
        None,
    )
    count => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::INT),
        &[
            Parameter {
                name: "sub".to_string(),
                ty: Type::str_ref(),
                has_default: false,
            },
            Parameter {
                name: "start".to_string(),
                ty: Type::int_ref(),
                has_default: true,
            },
            Parameter {
                name: "end".to_string(),
                ty: Type::int_ref(),
                has_default: true,
            },
        ],
        r#"Return the number of non-overlapping occurrences of substring sub in the range [start, end]. Optional arguments start and end are interpreted as in slice notation."#,
        false,
        None,
    )
    endswith => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::BOOL),
        &[
            Parameter {
                name: "val".to_string(),
                ty: Type::str_ref(),
                has_default: false,
            },
            Parameter {
                name: "start".to_string(),
                ty: Type::int_ref(),
                has_default: true,
            },
            Parameter {
                name: "end".to_string(),
                ty: Type::int_ref(),
                has_default: true,
            },
        ],
        r#"Return True if the string ends with the specified suffix, otherwise return False. suffix can also be a tuple of suffixes to look for. With optional start, test beginning at that position. With optional end, stop comparing at that position."#,
        false,
        None,
    )
    find => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::INT),
        &[
            Parameter {
                name: "sub".to_string(),
                ty: Type::str_ref(),
                has_default: false,
            },
            Parameter {
                name: "start".to_string(),
                ty: Type::int_ref(),
                has_default: true,
            },
            Parameter {
                name: "end".to_string(),
                ty: Type::int_ref(),
                has_default: true,
            },
        ],
        r#"Return the lowest index in the string where substring sub is found within the slice s[start:end]. Optional arguments start and end are interpreted as in slice notation. Return -1 if sub is not found."#,
        false,
        None,
    )
    format => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::STR),
        &[],
        r#"Perform a string formatting operation. The string on which this method is called can contain literal text or replacement fields delimited by braces {}. Each replacement field contains either the numeric index of a positional argument, or the name of a keyword argument. Returns a copy of the string where each replacement field is replaced with the string value of the corresponding argument."#,
        true,
        None,
    )
    index => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::INT),
        &[
            Parameter {
                name: "sub".to_string(),
                ty: Type::str_ref(),
                has_default: false,
            },
            Parameter {
                name: "start".to_string(),
                ty: Type::int_ref(),
                has_default: true,
            },
            Parameter {
                name: "end".to_string(),
                ty: Type::int_ref(),
                has_default: true,
            },
        ],
        r#"Like str.find(), but raise an error when the substring is not found."#,
        false,
        None,
    )
    isalpha => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::BOOL),
        &[],
        r#"Return True if all characters in the string are alphabetic and there is at least one character, False otherwise. Alphabetic characters are those characters defined in the Unicode character database as “Letter”, i.e., those with general category property being one of “Lm”, “Lt”, “Lu”, “Ll”, or “Lo”. Note that this is different from the “Alphabetic” property defined in the Unicode Standard."#,
        false,
        None,
    )
    isalnum => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::BOOL),
        &[],
        r#"Return True if all characters in the string are alphanumeric and there is at least one character, False otherwise. A character c is alphanumeric if one of the following returns True: c.isalpha(), c.isdecimal(), c.isdigit(), or c.isnumeric()."#,
        false,
        None,
    )
    isdigit => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::BOOL),
        &[],
        r#"Return True if all characters in the string are digits and there is at least one character, False otherwise. Digits include decimal characters and digits that need special handling, such as the compatibility superscript digits. This covers digits which cannot be used to form numbers in base 10, like the Kharosthi numbers. Formally, a digit is a character that has the property value Numeric_Type=Digit or Numeric_Type=Decimal."#,
        false,
        None,
    )
    islower => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::BOOL),
        &[],
        r#"Return True if all cased characters in the string are lowercase and there is at least one cased character, False otherwise."#,
        false,
        None,
    )
    isspace => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::BOOL),
        &[],
        r#"Return True if there are only whitespace characters in the string and there is at least one character, False otherwise."#,
        false,
        None,
    )
    istitle => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::BOOL),
        &[],
        r#"Return True if the string is a titlecased string and there is at least one character, for example uppercase characters may only follow uncased characters and lowercase characters only cased ones. Return False otherwise."#,
        false,
        None,
    )
    isupper => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::BOOL),
        &[],
        r#"Return True if all cased characters in the string are uppercase and there is at least one cased character, False otherwise."#,
        false,
        None,
    )
    join => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::STR),
        &[
            Parameter {
                name: "iter".to_string(),
                ty: Type::list_ref(Type::any_ref()),
                has_default: false,
            },
        ],
        r#"Return a string which is the concatenation of the strings in iterable. An error will be raised if there are any non-string values in iterable. The separator between elements is the string providing this method."#,
        false,
        None,
    )
    lower => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::STR),
        &[],
        r#"Return a copy of the string with all the cased characters converted to lowercase."#,
        false,
        None,
    )
    upper => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::STR),
        &[],
        r#"Return a copy of the string with all the cased characters converted to uppercase."#,
        false,
        None,
    )
    lstrip => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::STR),
        &[
            Parameter {
                name: "chars".to_string(),
                ty: Type::str_ref(),
                has_default: true,
            },
        ],
        r#"Return a copy of the string with leading characters removed. The chars argument is a string specifying the set of characters to be removed. If omitted or None, the chars argument defaults to removing whitespace. The chars argument is not a prefix; rather, all combinations of its values are stripped:"#,
        false,
        None,
    )
    rstrip => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::STR),
        &[
            Parameter {
                name: "chars".to_string(),
                ty: Type::str_ref(),
                has_default: true,
            },
        ],
        r#"Return a copy of the string with trailing characters removed. The chars argument is a string specifying the set of characters to be removed. If omitted or None, the chars argument defaults to removing whitespace. The chars argument is not a suffix; rather, all combinations of its values are stripped:"#,
        false,
        None,
    )
    replace => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::STR),
        &[
            Parameter {
                name: "old".to_string(),
                ty: Type::str_ref(),
                has_default: false,
            },
            Parameter {
                name: "new".to_string(),
                ty: Type::str_ref(),
                has_default: false,
            },
            Parameter {
                name: "count".to_string(),
                ty: Type::int_ref(),
                has_default: true,
            },
        ],
        r#"Return a copy of the string with all occurrences of substring old replaced by new. If the optional argument count is given, only the first count occurrences are replaced.Return a copy of the string with all occurrences of substring old replaced by new. If the optional argument count is given, only the first count occurrences are replaced."#,
        false,
        None,
    )
    removeprefix => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::STR),
        &[
            Parameter {
                name: "prefix".to_string(),
                ty: Type::str_ref(),
                has_default: false,
            },
        ],
        r#"If the string starts with the prefix string, return string[len(prefix):]. Otherwise, return a copy of the original string."#,
        false,
        None,
    )
    removesuffix => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::STR),
        &[
            Parameter {
                name: "suffix".to_string(),
                ty: Type::str_ref(),
                has_default: false,
            },
        ],
        r#"If the string ends with the suffix string and that suffix is not empty, return string[:-len(suffix)]. Otherwise, return a copy of the original string."#,
        false,
        None,
    )
    rfind => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::INT),
        &[
            Parameter {
                name: "sub".to_string(),
                ty: Type::str_ref(),
                has_default: false,
            },
            Parameter {
                name: "start".to_string(),
                ty: Type::int_ref(),
                has_default: true,
            },
            Parameter {
                name: "end".to_string(),
                ty: Type::int_ref(),
                has_default: true,
            },
        ],
        r#"Return the highest index in the string where substring sub is found, such that sub is contained within s[start:end]. Optional arguments start and end are interpreted as in slice notation. Return -1 on failure."#,
        false,
        None,
    )
    rindex => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::INT),
        &[
            Parameter {
                name: "sub".to_string(),
                ty: Type::str_ref(),
                has_default: false,
            },
            Parameter {
                name: "start".to_string(),
                ty: Type::int_ref(),
                has_default: true,
            },
            Parameter {
                name: "end".to_string(),
                ty: Type::int_ref(),
                has_default: true,
            },
        ],
        r#"Like rfind() but raises ValueError when the substring sub is not found."#,
        false,
        None,
    )
    rsplit => Type::function(
        Some(Arc::new(Type::STR)),
        Type::list_ref(Arc::new(Type::STR)),
        &[
            Parameter {
                name: "sep".to_string(),
                ty: Type::str_ref(),
                has_default: true,
            },
            Parameter {
                name: "maxsplit".to_string(),
                ty: Type::int_ref(),
                has_default: true,
            },
        ],
        r#"Return a list of the words in the string, using sep as the delimiter string. If maxsplit is given, at most maxsplit splits are done, the rightmost ones. If sep is not specified or None, any whitespace string is a separator. Except for splitting from the right, rsplit() behaves like split() which is described in detail below."#,
        false,
        None,
    )
    split => Type::function(
        Some(Arc::new(Type::STR)),
        Type::list_ref(Arc::new(Type::STR)),
        &[
            Parameter {
                name: "sep".to_string(),
                ty: Type::str_ref(),
                has_default: true,
            },
            Parameter {
                name: "maxsplit".to_string(),
                ty: Type::int_ref(),
                has_default: true,
            },
        ],
        r#"Return a list of the words in the string, using sep as the delimiter string. If maxsplit is given, at most maxsplit splits are done (thus, the list will have at most maxsplit+1 elements). If maxsplit is not specified or -1, then there is no limit on the number of splits (all possible splits are made)."#,
        false,
        None,
    )
    splitlines => Type::function(
        Some(Arc::new(Type::STR)),
        Type::list_ref(Arc::new(Type::STR)),
        &[
            Parameter {
                name: "keepends".to_string(),
                ty: Type::bool_ref(),
                has_default: true,
            },
        ],
        r#"Return a list of the lines in the string, breaking at line boundaries. Line breaks are not included in the resulting list unless keepends is given and true."#,
        false,
        None,
    )
    startswith => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::BOOL),
        &[
            Parameter {
                name: "val".to_string(),
                ty: Type::str_ref(),
                has_default: false,
            },
            Parameter {
                name: "start".to_string(),
                ty: Type::int_ref(),
                has_default: true,
            },
            Parameter {
                name: "end".to_string(),
                ty: Type::int_ref(),
                has_default: true,
            },
        ],
        r#"Return True if string starts with the prefix, otherwise return False. prefix can also be a tuple of prefixes to look for. With optional start, test string beginning at that position. With optional end, stop comparing string at that position."#,
        false,
        None,
    )
    strip => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::STR),
        &[
            Parameter {
                name: "chars".to_string(),
                ty: Type::str_ref(),
                has_default: true,
            },
        ],
        r#"Return a copy of the string with the leading and trailing characters removed. The chars argument is a string specifying the set of characters to be removed. If omitted or None, the chars argument defaults to removing whitespace. The chars argument is not a prefix or suffix; rather, all combinations of its values are stripped:"#,
        false,
        None,
    )
    title => Type::function(
        Some(Arc::new(Type::STR)),
        Arc::new(Type::STR),
        &[],
        r#"Return a titlecased version of the string where words start with an uppercase character and the remaining characters are lowercase."#,
        false,
        None,
    )
}
