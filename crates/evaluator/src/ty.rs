use kcl_runtime::{
    BUILTIN_TYPE_BOOL, BUILTIN_TYPE_FLOAT, BUILTIN_TYPE_INT, BUILTIN_TYPE_STR, BUILTIN_TYPES,
    ConfigEntryOperationKind, DictValue, KCL_TYPE_ANY, ListValue, PKG_PATH_PREFIX, Value, ValueRef,
    check_type, dereference_type, is_dict_type, is_list_type, is_schema_type, is_type_union,
    schema_config_meta, schema_runtime_type, separate_kv, split_type_union, val_plan,
};
use scopeguard::defer;

use crate::error as kcl_error;
use crate::schema::SchemaEvalContext;
use crate::{Evaluator, proxy::Proxy};

/// Use the schema instance to build a new schema instance using the schema construct function
pub fn resolve_schema(s: &Evaluator, schema: &ValueRef, keys: &[String]) -> ValueRef {
    if !schema.is_schema() {
        return schema.clone();
    }
    let schema_value = schema.as_schema();
    let schema_type_name = schema_runtime_type(&schema_value.name, &schema_value.pkgpath);
    if let Some(index) = s.schemas.borrow().get(&schema_type_name) {
        let keys = keys.iter().map(|v| v.as_str()).collect();
        let config_value =
            schema.dict_get_entries_with_op(keys, &ConfigEntryOperationKind::Override);
        let config_meta = {
            let ctx = s.runtime_ctx.borrow();
            schema_config_meta(
                &ctx.panic_info.kcl_file,
                ctx.panic_info.kcl_line as u64,
                ctx.panic_info.kcl_col as u64,
            )
        };

        let frame = {
            let frames = s.frames.borrow();
            frames
                .get(*index)
                .expect(kcl_error::INTERNAL_ERROR_MSG)
                .clone()
        };
        let schema = if let Proxy::Schema(caller) = &frame.proxy {
            s.push_pkgpath(&frame.pkgpath);
            s.push_backtrace(&frame);
            defer! {
                s.pop_backtrace();
                s.pop_pkgpath();
            }

            (caller.body)(
                s,
                &caller.ctx.borrow().snapshot(config_value, config_meta),
                &schema_value.args,
                &schema_value.kwargs,
            )
        } else {
            schema.clone()
        };
        return schema;
    }
    schema.clone()
}

/// Type pack and check ValueRef with the expected type vector
pub fn type_pack_and_check(
    s: &Evaluator,
    value: &ValueRef,
    expected_types: Vec<&str>,
    strict: bool,
) -> ValueRef {
    if value.is_none_or_undefined() || expected_types.is_empty() {
        return value.clone();
    }
    let is_schema = value.is_schema();
    let mut checked = false;
    let mut converted_value = value.clone();
    for tpe in &expected_types {
        // Types that cannot request schema conversion can be checked before
        // copying their containers. Keep top-level `any` on the generic path
        // for its required-attribute validation below.
        if !is_schema
            && let Some(plain) = PlainType::parse(tpe)
            && plain != PlainType::Any
        {
            if plain.check(
                value,
                &s.runtime_ctx.borrow().panic_info.kcl_pkgpath,
                strict,
            ) {
                converted_value = plain.convert(value);
                checked = true;
                break;
            }
            continue;
        }
        if !is_schema {
            converted_value = convert_collection_value(s, value, tpe);
        }
        // Runtime type check
        checked = check_type(
            &converted_value,
            &s.runtime_ctx.borrow().panic_info.kcl_pkgpath,
            tpe,
            strict,
        );
        // For schema types, validate required attributes in specific cases:
        // 1. When the value is an empty dict (likely user error like User({}))
        // 2. When the value is a standalone dict (not part of a config merge)
        if checked && is_schema_type(tpe) && value.is_config() {
            let config = value.as_dict_ref();
            // Only validate if it looks like a user-provided schema instance:
            // - Empty dict (definitely missing required attributes)
            // - Dict with some attributes but not all (partial definition)
            // Skip validation for config merge scenarios (inherited/mixin attributes)
            if config.values.is_empty() {
                // Empty dict - definitely needs validation
                let schema_type_name = if tpe.contains('.') {
                    if tpe.starts_with(PKG_PATH_PREFIX) {
                        tpe.to_string()
                    } else {
                        format!("{PKG_PATH_PREFIX}{tpe}")
                    }
                } else {
                    format!("{}.{}", s.current_pkgpath(), tpe)
                };

                if let Some(index) = s.schemas.borrow().get(&schema_type_name) {
                    let frame = {
                        let frames = s.frames.borrow();
                        frames
                            .get(*index)
                            .expect(kcl_error::INTERNAL_ERROR_MSG)
                            .clone()
                    };
                    if let Proxy::Schema(caller) = &frame.proxy {
                        for (attr, _is_optional) in SchemaEvalContext::get_attrs(s, &caller.ctx) {
                            let is_required =
                                SchemaEvalContext::is_attr_required(s, &caller.ctx, &attr);
                            if is_required {
                                // Required attribute is missing, set checked to false
                                checked = false;
                                break;
                            }
                        }
                    }
                }
            }
        }
        if checked {
            break;
        }
    }
    if !checked {
        let mut error_msgs = vec![];
        for tpe in &expected_types {
            if is_schema_type(tpe) {
                let schema_type_name = if tpe.contains('.') {
                    if tpe.starts_with(PKG_PATH_PREFIX) {
                        tpe.to_string()
                    } else {
                        format!("{PKG_PATH_PREFIX}{tpe}")
                    }
                } else {
                    format!("{}.{}", s.current_pkgpath(), tpe)
                };

                if let Some(index) = s.schemas.borrow().get(&schema_type_name) {
                    let frame = {
                        let frames = s.frames.borrow();
                        frames
                            .get(*index)
                            .expect(kcl_error::INTERNAL_ERROR_MSG)
                            .clone()
                    };
                    if let Proxy::Schema(caller) = &frame.proxy
                        && value.is_config()
                    {
                        let config = value.as_dict_ref();
                        for (key, _) in &config.values {
                            let no_such_attr = !SchemaEvalContext::has_attr(s, &caller.ctx, key)
                                && !key.starts_with('_');
                            let has_index_signature =
                                SchemaEvalContext::has_index_signature(s, &caller.ctx);
                            if !has_index_signature && no_such_attr {
                                error_msgs.push(format!(
                                    "Schema {} does not contain attribute {}",
                                    tpe, key
                                ));
                            }
                        }

                        for (attr, is_optional) in SchemaEvalContext::get_attrs(s, &caller.ctx) {
                            if !config.values.contains_key(attr.as_str()) && !is_optional {
                                error_msgs.push(format!(
                                    "Schema {}'s attribute {} is missing",
                                    tpe, attr
                                ));
                            }
                        }
                    }
                }
            }
        }
        let expected_type = expected_types.join(" | ").replace('@', "");
        panic!(
            "expect {expected_type}, got {}{}",
            val_plan::type_of(value, true),
            if error_msgs.is_empty() {
                "".to_string()
            } else {
                format!(". For details:\n{}", error_msgs.join("\n"))
            }
        );
    }
    converted_value
}

/// A type whose [`convert_collection_value`] can never produce a schema: `any`,
/// a builtin type, or a list or dict of such types, e.g. `[{str:str}]`.
///
/// For these types the conversion only copies the list and dict containers, so
/// [`PlainType::check`] and [`PlainType::convert`] do the same work as
/// [`check_type`] and [`convert_collection_value`] without building a type
/// string for every element. Any other type string, including unions, literal
/// types, schema types and strings with spaces, is not parsed and keeps the
/// generic path.
#[derive(Debug, PartialEq)]
enum PlainType<'a> {
    /// `any` or the empty type.
    Any,
    /// `bool`, `int`, `float` or `str`.
    Builtin(&'a str),
    /// `[T]`, with its type string and element type.
    List(&'a str, Box<PlainType<'a>>),
    /// `{K:T}`, with its type string and value type. Like [`check_type`], the
    /// key type is not checked.
    Dict(&'a str, Box<PlainType<'a>>),
}

impl<'a> PlainType<'a> {
    fn parse(tpe: &'a str) -> Option<Self> {
        if tpe.is_empty() || tpe == KCL_TYPE_ANY {
            Some(Self::Any)
        } else if BUILTIN_TYPES.contains(&tpe) {
            Some(Self::Builtin(tpe))
        } else if is_list_type(tpe) {
            let elem = Self::parse(&tpe[1..tpe.len() - 1])?;
            Some(Self::List(tpe, Box::new(elem)))
        } else if is_dict_type(tpe) {
            // Allowing only builtin or `any` key types keeps brackets out of the
            // key, so the first ':' is the one `separate_kv` splits on.
            let (key, value) = tpe[1..tpe.len() - 1].split_once(':')?;
            if key != KCL_TYPE_ANY && !BUILTIN_TYPES.contains(&key) {
                return None;
            }
            Some(Self::Dict(tpe, Box::new(Self::parse(value)?)))
        } else {
            None
        }
    }

    /// Same result as `check_type(value, pkgpath, tpe, strict)`.
    fn check(&self, value: &ValueRef, pkgpath: &str, strict: bool) -> bool {
        let tpe = match self {
            Self::Any => return true,
            Self::Builtin(tpe) | Self::List(tpe, _) | Self::Dict(tpe, _) => *tpe,
        };
        match (&*value.rc.borrow(), self) {
            (Value::none | Value::undefined, _) => true,
            (Value::bool_value(_), Self::Builtin(tpe)) => *tpe == BUILTIN_TYPE_BOOL,
            (Value::int_value(_), Self::Builtin(tpe)) => {
                *tpe == BUILTIN_TYPE_INT || *tpe == BUILTIN_TYPE_FLOAT
            }
            (Value::float_value(_), Self::Builtin(tpe)) => *tpe == BUILTIN_TYPE_FLOAT,
            (Value::str_value(_), Self::Builtin(tpe)) => *tpe == BUILTIN_TYPE_STR,
            (Value::list_value(list), Self::List(_, elem)) => {
                list.values.iter().all(|v| elem.check(v, pkgpath, false))
            }
            (Value::dict_value(dict), Self::Dict(_, elem)) => {
                dict.values.values().all(|v| elem.check(v, pkgpath, false))
            }
            (
                Value::bool_value(_)
                | Value::int_value(_)
                | Value::float_value(_)
                | Value::str_value(_)
                | Value::list_value(_)
                | Value::dict_value(_),
                _,
            ) => false,
            // Units, functions and schemas.
            _ => check_type(value, pkgpath, tpe, strict),
        }
    }

    /// Same result as `convert_collection_value(s, value, tpe)`: a new list or
    /// dict for every container the type describes, sharing everything else.
    fn convert(&self, value: &ValueRef) -> ValueRef {
        match (self, &*value.rc.borrow()) {
            (Self::List(_, elem), Value::list_value(list)) => {
                ValueRef::from(Value::list_value(Box::new(ListValue {
                    values: list.values.iter().map(|v| elem.convert(v)).collect(),
                })))
            }
            (Self::Dict(_, elem), Value::dict_value(dict)) => {
                let mut result = DictValue::default();
                for (k, v) in &dict.values {
                    result.values.insert(k.clone(), elem.convert(v));
                    let op = dict.ops.get(k).unwrap_or(&ConfigEntryOperationKind::Union);
                    result.ops.insert(k.clone(), op.clone());
                    if let Some(index) = dict.insert_indexs.get(k) {
                        result.insert_indexs.insert(k.clone(), *index);
                    }
                }
                result.potential_schema = dict
                    .potential_schema
                    .as_ref()
                    .filter(|ty| !ty.is_empty())
                    .cloned();
                ValueRef::from(Value::dict_value(Box::new(result)))
            }
            _ => value.clone(),
        }
    }
}

/// Convert collection value including dict/list to the potential schema
pub fn convert_collection_value(s: &Evaluator, value: &ValueRef, tpe: &str) -> ValueRef {
    if tpe.is_empty() || tpe == KCL_TYPE_ANY {
        return value.clone();
    }
    let is_collection = value.is_list() || value.is_dict();
    let invalid_match_dict = is_dict_type(tpe) && !value.is_dict();
    let invalid_match_list = is_list_type(tpe) && !value.is_list();
    let invalid_match = invalid_match_dict || invalid_match_list;
    if !is_collection || invalid_match {
        return value.clone();
    }
    // Convert a value to union types e.g., {a: 1} => A | B
    if is_type_union(tpe) {
        let types = split_type_union(tpe);
        convert_collection_value_with_union_types(s, value, &types)
    } else if is_dict_type(tpe) {
        //let (key_tpe, value_tpe) = separate_kv(tpe);
        let (_, value_tpe) = separate_kv(&dereference_type(tpe));
        let mut expected_dict = ValueRef::dict(None);
        let dict_ref = value.as_dict_ref();
        expected_dict
            .set_potential_schema_type(&dict_ref.potential_schema.clone().unwrap_or_default());
        for (k, v) in &dict_ref.values {
            let expected_value = convert_collection_value(s, v, &value_tpe);
            let op = dict_ref
                .ops
                .get(k)
                .unwrap_or(&ConfigEntryOperationKind::Union);
            let index = dict_ref.insert_indexs.get(k);
            expected_dict.dict_update_entry(k, &expected_value, op, index)
        }
        expected_dict
    } else if is_list_type(tpe) {
        let expected_type = dereference_type(tpe);
        let mut expected_list = ValueRef::list(None);
        let list_ref = value.as_list_ref();
        for v in &list_ref.values {
            let expected_value = convert_collection_value(s, v, &expected_type);
            expected_list.list_append(&expected_value)
        }
        expected_list
    } else if BUILTIN_TYPES.contains(&tpe) {
        value.clone()
    } else {
        // Get the type form @pkg.Schema
        let schema_type_name = if tpe.contains('.') {
            if tpe.starts_with(PKG_PATH_PREFIX) {
                tpe.to_string()
            } else {
                format!("{PKG_PATH_PREFIX}{tpe}")
            }
        } else {
            format!("{}.{}", s.current_pkgpath(), tpe)
        };
        if let Some(index) = s.schemas.borrow().get(&schema_type_name) {
            let config_meta = {
                let ctx = s.runtime_ctx.borrow();
                schema_config_meta(
                    &ctx.panic_info.kcl_file,
                    ctx.panic_info.kcl_line as u64,
                    ctx.panic_info.kcl_col as u64,
                )
            };
            let frame = {
                let frames = s.frames.borrow();
                frames
                    .get(*index)
                    .expect(kcl_error::INTERNAL_ERROR_MSG)
                    .clone()
            };
            let schema = if let Proxy::Schema(caller) = &frame.proxy {
                // Try convert the config to schema, if failed, return the config directly.
                if !SchemaEvalContext::is_fit_config(s, &caller.ctx, value) {
                    return value.clone();
                }
                s.push_pkgpath(&frame.pkgpath);
                s.push_backtrace(&frame);
                let value = (caller.body)(
                    s,
                    &caller.ctx.borrow().snapshot(value.clone(), config_meta),
                    &s.list_value(),
                    &s.dict_value(),
                );
                s.pop_backtrace();
                s.pop_pkgpath();
                value
            } else {
                value.clone()
            };
            // ctx.panic_info = now_panic_info;
            return schema.clone();
        }
        // ctx.panic_info = now_meta_info;
        value.clone()
    }
}

/// Convert collection value including dict/list to the potential schema and return errors.
pub fn convert_collection_value_with_union_types(
    s: &Evaluator,
    value: &ValueRef,
    types: &[&str],
) -> ValueRef {
    if value.is_schema() {
        value.clone()
    } else {
        for tpe in types {
            // Try match every type and convert the value, if matched, return the value.
            let value = convert_collection_value(s, value, tpe);
            if check_type(
                &value,
                &s.runtime_ctx.borrow().panic_info.kcl_pkgpath,
                tpe,
                false,
            ) {
                return value;
            }
        }
        value.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kcl_loader::{LoadPackageOptions, load_packages};
    use kcl_parser::LoadProgramOptions;
    use kcl_runtime::MAIN_PKG_PATH;

    fn with_evaluator(f: impl FnOnce(&Evaluator)) {
        let p = load_packages(&LoadPackageOptions {
            paths: vec!["test.k".to_string()],
            load_opts: Some(LoadProgramOptions {
                k_code_list: vec!["a = 1".to_string()],
                ..Default::default()
            }),
            load_builtin: false,
            ..Default::default()
        })
        .unwrap();
        f(&Evaluator::new(&p.program));
    }

    fn list(values: &[ValueRef]) -> ValueRef {
        ValueRef::list_value(Some(values))
    }

    fn dict(entries: &[(&str, ValueRef)]) -> ValueRef {
        let entries: Vec<(&str, &ValueRef)> = entries.iter().map(|(k, v)| (*k, v)).collect();
        ValueRef::dict(Some(&entries))
    }

    /// Values covering every arm of `PlainType::check` and `PlainType::convert`.
    fn sample_values() -> Vec<ValueRef> {
        // A dict with entry operators, insert indexes and a potential schema.
        let mut with_meta = ValueRef::dict(None);
        with_meta.dict_update_entry(
            "a",
            &list(&[ValueRef::int(1)]),
            &ConfigEntryOperationKind::Insert,
            Some(&0),
        );
        with_meta.dict_update_entry(
            "b",
            &list(&[ValueRef::int(2)]),
            &ConfigEntryOperationKind::Override,
            None,
        );
        with_meta.set_potential_schema_type("pkg.Person");
        if let Value::dict_value(d) = &mut *with_meta.rc.borrow_mut() {
            d.ops.shift_remove("b");
            d.ops
                .insert("unused".into(), ConfigEntryOperationKind::Insert);
            d.insert_indexs.insert("unused".into(), -1);
            d.attr_map.insert("a".into(), "[int]".to_string());
            d.schema_args = Some((list(&[ValueRef::int(1)]), dict(&[])));
        }
        let empty_potential_schema = dict(&[("a", ValueRef::str("v"))]);
        if let Value::dict_value(d) = &mut *empty_potential_schema.rc.borrow_mut() {
            d.potential_schema = Some(String::new());
        }
        vec![
            ValueRef::none(),
            ValueRef::undefined(),
            ValueRef::bool(true),
            ValueRef::int(1),
            ValueRef::float(1.5),
            ValueRef::str("s"),
            ValueRef::unit(1024.0, 1, "Ki"),
            ValueRef::schema(),
            ValueRef::func(0, 0, ValueRef::none(), "f", "", false),
            ValueRef::func(0, 0, ValueRef::none(), "S", "pkg.S", false),
            list(&[]),
            list(&[ValueRef::int(1), ValueRef::float(2.5), ValueRef::none()]),
            list(&[ValueRef::int(1), ValueRef::str("s")]),
            list(&[ValueRef::bool(false), ValueRef::undefined()]),
            list(&[list(&[ValueRef::int(1)]), list(&[])]),
            list(&[
                dict(&[("k", ValueRef::str("v"))]),
                dict(&[("k", ValueRef::none())]),
            ]),
            list(&[dict(&[("k", ValueRef::int(1))])]),
            list(&[ValueRef::schema()]),
            list(&[ValueRef::unit(1024.0, 1, "Ki")]),
            dict(&[]),
            dict(&[("k", ValueRef::str("v")), ("j", ValueRef::str("w"))]),
            dict(&[("k", ValueRef::int(1))]),
            dict(&[("k", list(&[ValueRef::int(1), ValueRef::int(2)]))]),
            dict(&[("k", dict(&[("a", ValueRef::str("b"))]))]),
            dict(&[("k", ValueRef::schema())]),
            dict(&[("1", ValueRef::int(1))]),
            with_meta,
            empty_potential_schema,
        ]
    }

    const PLAIN_TYPES: &[&str] = &[
        "",
        "any",
        "int",
        "float",
        "bool",
        "str",
        "[]",
        "[int]",
        "[float]",
        "[str]",
        "[bool]",
        "[any]",
        "[[int]]",
        "[{str:str}]",
        "{str:str}",
        "{str:int}",
        "{str:any}",
        "{str:}",
        "{any:str}",
        "{bool:int}",
        "{float:int}",
        "{int:[int]}",
        "{str:[int]}",
        "{str:{str:str}}",
        "[{str:[float]}]",
    ];

    /// Asserts that `a` and `b` share exactly the values they share with
    /// `orig`, and otherwise hold the same entries, entry operators, insert
    /// indexes and metadata in the same order.
    fn assert_same_conversion(orig: &ValueRef, a: &ValueRef, b: &ValueRef, tpe: &str) {
        assert_eq!(
            a.is_same_ref(orig),
            b.is_same_ref(orig),
            "sharing differs for {tpe}: {a:?} vs {b:?}"
        );
        match (&*orig.rc.borrow(), &*a.rc.borrow(), &*b.rc.borrow()) {
            (Value::list_value(o), Value::list_value(x), Value::list_value(y)) => {
                assert_eq!(x.values.len(), y.values.len(), "{tpe}");
                for ((o, x), y) in o.values.iter().zip(&x.values).zip(&y.values) {
                    assert_same_conversion(o, x, y, tpe);
                }
            }
            (Value::dict_value(o), Value::dict_value(x), Value::dict_value(y)) => {
                let keys = |d: &DictValue| d.values.keys().cloned().collect::<Vec<_>>();
                let ops = |d: &DictValue| d.ops.clone().into_iter().collect::<Vec<_>>();
                let indexes =
                    |d: &DictValue| d.insert_indexs.clone().into_iter().collect::<Vec<_>>();
                let attrs = |d: &DictValue| d.attr_map.clone().into_iter().collect::<Vec<_>>();
                assert_eq!(keys(x), keys(y), "{tpe}");
                assert_eq!(ops(x), ops(y), "{tpe}");
                assert_eq!(indexes(x), indexes(y), "{tpe}");
                assert_eq!(attrs(x), attrs(y), "{tpe}");
                assert_eq!(x.potential_schema, y.potential_schema, "{tpe}");
                assert_eq!(x.schema_args.is_none(), y.schema_args.is_none(), "{tpe}");
                for (k, o) in &o.values {
                    assert_same_conversion(o, &x.values[k], &y.values[k], tpe);
                }
            }
            _ => assert!(a.is_same_ref(b), "{tpe}: {a:?} vs {b:?}"),
        }
    }

    #[test]
    fn plain_type_parse() {
        for tpe in PLAIN_TYPES {
            assert!(PlainType::parse(tpe).is_some(), "{tpe} should parse");
        }
        assert_eq!(PlainType::parse(""), Some(PlainType::Any));
        assert_eq!(PlainType::parse("any"), Some(PlainType::Any));
        assert_eq!(
            PlainType::parse("[{str:int}]"),
            Some(PlainType::List(
                "[{str:int}]",
                Box::new(PlainType::Dict(
                    "{str:int}",
                    Box::new(PlainType::Builtin("int"))
                ))
            ))
        );
        // Types that may convert to a schema, unions, literal and number
        // multiplier types, and type strings written in another form.
        for tpe in [
            "Person",
            "pkg.Person",
            "@pkg.Person",
            "pkg.int",
            "@pkg.int",
            "Int",
            "[Person]",
            "{str:Person}",
            "{str:[Person]}",
            "int | str",
            "int|str",
            "[int | str]",
            "[int]|[str]",
            "{str:int}|{str:str}",
            "[\"a\"]",
            "\"a\"",
            "1",
            "1.5",
            "True",
            "None",
            "1Ki",
            "units.NumberMultiplier",
            "(int) -> int",
            "{str: str}",
            "[ int]",
            "[int ]",
            "[\tint]",
            " int",
            "int ",
            "{\"a\":str}",
            "{[str]:str}",
            "{:str}",
            "{str}",
            "[",
            "{",
            "list",
            "dict",
        ] {
            assert_eq!(PlainType::parse(tpe), None, "{tpe} should not parse");
        }
    }

    #[test]
    fn plain_type_matches_check_type_and_conversion() {
        with_evaluator(|s| {
            for tpe in PLAIN_TYPES {
                let plain = PlainType::parse(tpe).unwrap();
                for value in sample_values() {
                    for strict in [false, true] {
                        let expected = check_type(&value, MAIN_PKG_PATH, tpe, strict);
                        assert_eq!(
                            plain.check(&value, MAIN_PKG_PATH, strict),
                            expected,
                            "check {value:?} against {tpe} (strict: {strict})"
                        );
                        if expected {
                            let actual = type_pack_and_check(s, &value, vec![tpe], strict);
                            let converted = convert_collection_value(s, &value, tpe);
                            assert_same_conversion(&value, &actual, &converted, tpe);
                        } else {
                            let error =
                                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                    type_pack_and_check(s, &value, vec![tpe], strict)
                                }))
                                .expect_err("expected a runtime type error");
                            assert_eq!(
                                error.downcast_ref::<String>().unwrap(),
                                &format!("expect {tpe}, got {}", val_plan::type_of(&value, true))
                            );
                        }
                    }
                    let converted = plain.convert(&value);
                    let expected = convert_collection_value(s, &value, tpe);
                    assert_same_conversion(&value, &converted, &expected, tpe);
                }
            }
        });
    }

    #[test]
    fn plain_type_preserves_repeated_reference_sharing() {
        with_evaluator(|s| {
            let inner = list(&[ValueRef::int(1)]);
            let value = list(&[inner.clone(), inner]);
            for tpe in ["any", "[any]", "[[int]]"] {
                let actual = type_pack_and_check(s, &value, vec![tpe], false);
                let expected = convert_collection_value(s, &value, tpe);
                assert_same_conversion(&value, &actual, &expected, tpe);
                let a = actual.as_list_ref();
                let b = expected.as_list_ref();
                assert_eq!(
                    a.values[0].is_same_ref(&a.values[1]),
                    b.values[0].is_same_ref(&b.values[1]),
                    "{tpe}"
                );
            }
        });
    }

    #[test]
    fn plain_type_falls_through_to_other_expected_types() {
        with_evaluator(|s| {
            for types in [
                vec!["[int]", "[str]"],
                vec!["[str]", "any"],
                vec!["[str]", "[int|str]"],
                vec!["[str]", "[int | str]"],
                vec!["[str]", "[1]"],
                vec!["[str]", "[units.NumberMultiplier]"],
                vec!["[str]", "[1Ki]"],
            ] {
                for value in [
                    list(&[ValueRef::int(1)]),
                    list(&[ValueRef::str("s")]),
                    list(&[ValueRef::unit(1024.0, 1, "Ki")]),
                ] {
                    if let Some(expected) = types.iter().find_map(|tpe| {
                        let converted = convert_collection_value(s, &value, tpe);
                        check_type(&converted, MAIN_PKG_PATH, tpe, false).then_some(converted)
                    }) {
                        let actual = type_pack_and_check(s, &value, types.clone(), false);
                        assert_same_conversion(&value, &actual, &expected, &types.join(" | "));
                    }
                }
            }
        });
    }
}
