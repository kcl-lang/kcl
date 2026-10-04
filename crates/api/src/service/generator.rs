//! Emitters backing the generator RPCs (`GenerateToml`, `GenerateKcl`).
//!
//! Two small, deterministic conversion pipelines live here:
//!
//! * [`yaml_to_toml`] — converts the YAML intermediate produced by
//!   `exec_program` into a [`toml::Value`] document, and
//! * [`value_to_kcl`] — renders a parsed data document (JSON, YAML or TOML)
//!   as KCL source with fixed formatting rules (4-space indents, inline
//!   scalars-only arrays up to 80 columns).
//!
//! Both operate on [`serde_yaml::Value`] because its `Mapping` preserves
//! insertion order (indexmap-backed), so source key order survives into the
//! output whenever the target format allows it.

use anyhow::Context;

/// Data formats accepted by the `GenerateKcl` RPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DataFormat {
    Json,
    Yaml,
    Toml,
}

impl DataFormat {
    fn from_name(name: &str) -> Option<DataFormat> {
        match name {
            "json" => Some(DataFormat::Json),
            "yaml" | "yml" => Some(DataFormat::Yaml),
            "toml" => Some(DataFormat::Toml),
            _ => None,
        }
    }
}

/// Resolve the data format for `GenerateKcl`: an explicit `format` argument
/// wins (validated case-insensitively against `json`/`yaml`/`toml`), then the
/// `filename` extension, defaulting to JSON when there is none.
pub(crate) fn resolve_data_format(explicit: &str, filename: &str) -> anyhow::Result<DataFormat> {
    let name = explicit.trim().to_lowercase();
    if !name.is_empty() {
        return DataFormat::from_name(&name).ok_or_else(|| {
            anyhow::anyhow!(
                "unsupported data format {name:?}; expected one of \"json\", \"yaml\" or \"toml\""
            )
        });
    }
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());
    Ok(match ext.as_deref() {
        Some(name) => DataFormat::from_name(name).unwrap_or(DataFormat::Json),
        None => DataFormat::Json,
    })
}

/// Parse `source` data content into a [`serde_yaml::Value`]. JSON is parsed
/// through the YAML reader (JSON is a YAML 1.2 subset); TOML is parsed with
/// the `toml` crate and converted, preserving key order.
pub(crate) fn parse_data(format: DataFormat, source: &str) -> anyhow::Result<serde_yaml::Value> {
    if source.trim().is_empty() {
        anyhow::bail!("source is empty");
    }
    match format {
        DataFormat::Json | DataFormat::Yaml => {
            serde_yaml::from_str(source).context("failed to parse data content")
        }
        DataFormat::Toml => {
            let value: toml::Value =
                toml::from_str(source).context("failed to parse TOML data content")?;
            toml_to_yaml(value)
        }
    }
}

/// Convert a parsed TOML document into a [`serde_yaml::Value`], preserving
/// table key order. TOML datetimes become plain strings.
fn toml_to_yaml(value: toml::Value) -> anyhow::Result<serde_yaml::Value> {
    use toml::Value as T;
    Ok(match value {
        T::String(s) => serde_yaml::Value::String(s),
        T::Integer(i) => serde_yaml::Value::Number(i.into()),
        T::Float(f) => serde_yaml::Value::Number(f.into()),
        T::Boolean(b) => serde_yaml::Value::Bool(b),
        T::Datetime(d) => serde_yaml::Value::String(d.to_string()),
        T::Array(items) => serde_yaml::Value::Sequence(
            items
                .into_iter()
                .map(toml_to_yaml)
                .collect::<anyhow::Result<Vec<_>>>()?,
        ),
        T::Table(table) => {
            let mut mapping = serde_yaml::Mapping::new();
            for (k, v) in table {
                mapping.insert(serde_yaml::Value::String(k), toml_to_yaml(v)?);
            }
            serde_yaml::Value::Mapping(mapping)
        }
    })
}

/// Strip a YAML tag, keeping the inner value.
fn untag(value: &serde_yaml::Value) -> &serde_yaml::Value {
    match value {
        serde_yaml::Value::Tagged(tagged) => untag(&tagged.value),
        _ => value,
    }
}

/// Convert the YAML intermediate of an `exec_program` call into a TOML
/// document. `sort_keys` sorts mapping keys recursively by their rendered
/// string form before conversion; otherwise source order is preserved, except
/// that within each TOML table non-table values are emitted before sub-tables
/// (a hard requirement of the TOML serializer). Keys holding null values are
/// dropped (TOML has no null); a null *array element* is an error because it
/// cannot be dropped without shifting indices.
pub(crate) fn yaml_to_toml(
    value: &serde_yaml::Value,
    sort_keys: bool,
) -> anyhow::Result<toml::Value> {
    Ok(convert(value, sort_keys)?.unwrap_or_else(|| toml::Value::Table(toml::map::Map::new())))
}

/// Convert a YAML value to a TOML value; `Ok(None)` means "null — drop me".
fn convert(value: &serde_yaml::Value, sort_keys: bool) -> anyhow::Result<Option<toml::Value>> {
    use serde_yaml::Value as Y;
    match untag(value) {
        Y::Null => Ok(None),
        Y::Bool(b) => Ok(Some(toml::Value::Boolean(*b))),
        Y::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Some(toml::Value::Integer(i)))
            } else if let Some(u) = n.as_u64() {
                let i = i64::try_from(u)
                    .with_context(|| format!("integer {u} is out of range for TOML (i64)"))?;
                Ok(Some(toml::Value::Integer(i)))
            } else if let Some(f) = n.as_f64() {
                if !f.is_finite() {
                    anyhow::bail!("non-finite float {f} cannot be represented in TOML");
                }
                Ok(Some(toml::Value::Float(f)))
            } else {
                anyhow::bail!("unsupported number {n}")
            }
        }
        Y::String(s) => Ok(Some(toml::Value::String(s.clone()))),
        Y::Sequence(items) => {
            let mut array = Vec::with_capacity(items.len());
            for item in items {
                match convert(item, sort_keys)? {
                    Some(v) => array.push(v),
                    None => anyhow::bail!(
                        "null array element cannot be represented in TOML (TOML has no null)"
                    ),
                }
            }
            Ok(Some(toml::Value::Array(array)))
        }
        Y::Mapping(map) => {
            let mut entries: Vec<(String, &Y)> = Vec::with_capacity(map.len());
            for (k, v) in map {
                entries.push((toml_key(k)?, v));
            }
            if sort_keys {
                entries.sort_by(|a, b| a.0.cmp(&b.0));
            }
            let mut converted: Vec<(String, toml::Value)> = Vec::with_capacity(entries.len());
            for (key, v) in entries {
                // A null value drops the key from the TOML document.
                if let Some(v) = convert(v, sort_keys)? {
                    converted.push((key, v));
                }
            }
            let mut table = toml::map::Map::new();
            // TOML requires all non-table values of a table to be emitted
            // before any sub-table, so partition stably: values first, then
            // tables, each group keeping source (or sorted) order.
            let (values, tables): (Vec<_>, Vec<_>) = converted
                .into_iter()
                .partition(|(_, v)| !matches!(v, toml::Value::Table(_)));
            for (key, v) in values {
                table.insert(key, v);
            }
            for (key, v) in tables {
                table.insert(key, v);
            }
            Ok(Some(toml::Value::Table(table)))
        }
        other => anyhow::bail!("unsupported YAML value for TOML conversion: {other:?}"),
    }
}

/// Render a YAML mapping key for TOML: string keys pass through, other scalar
/// keys convert via their scalar rendering. Complex (sequence/mapping/null)
/// keys are rejected — TOML has no key type but strings.
fn toml_key(key: &serde_yaml::Value) -> anyhow::Result<String> {
    use serde_yaml::Value as Y;
    match untag(key) {
        Y::String(s) => Ok(s.clone()),
        Y::Bool(b) => Ok(b.to_string()),
        Y::Number(n) => render_number(n),
        other => {
            anyhow::bail!("unsupported TOML key {other:?}: keys must be strings or scalar values")
        }
    }
}

/// Render a KCL document from parsed data. The root must be a mapping, which
/// becomes the top-level KCL attributes; anything else is an error because
/// KCL programs are sequences of `key = value` statements.
pub(crate) fn value_to_kcl(value: &serde_yaml::Value) -> anyhow::Result<String> {
    let mut out = String::new();
    match untag(value) {
        serde_yaml::Value::Mapping(map) => {
            for (k, v) in map {
                let key = match untag(k) {
                    serde_yaml::Value::String(s) => s.clone(),
                    other => {
                        anyhow::bail!("data object keys must be strings, found key {other:?}")
                    }
                };
                out.push_str(&render_key(&key));
                out.push_str(" = ");
                emit_value(&mut out, v, 0)?;
                out.push('\n');
            }
            Ok(out)
        }
        other => anyhow::bail!(
            "the data root must be an object (mapping) so it can become top-level KCL attributes, found {other:?}"
        ),
    }
}

/// Keys are bare when they form a valid KCL identifier, double-quoted
/// otherwise.
fn render_key(key: &str) -> String {
    let bare = match key.chars().next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {
            key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        _ => false,
    };
    if bare {
        key.to_string()
    } else {
        serde_json::to_string(key).expect("serializing a string to JSON cannot fail")
    }
}

/// Render a float with the shortest round-trip representation (`f64`'s
/// `Display`), keeping a trailing `.0` for integral values so `1.0` does not
/// collapse into the integer-looking `1`. `Display` never emits an exponent,
/// so no further normalization is needed.
fn render_float(f: f64) -> anyhow::Result<String> {
    if !f.is_finite() {
        anyhow::bail!("non-finite float {f} cannot be represented in KCL");
    }
    let mut s = f.to_string();
    if f.fract() == 0.0 && !s.contains('.') {
        s.push_str(".0");
    }
    Ok(s)
}

/// Render an integer or float YAML number.
fn render_number(n: &serde_yaml::Number) -> anyhow::Result<String> {
    if let Some(i) = n.as_i64() {
        Ok(i.to_string())
    } else if let Some(u) = n.as_u64() {
        Ok(u.to_string())
    } else if let Some(f) = n.as_f64() {
        render_float(f)
    } else {
        anyhow::bail!("unsupported number {n}")
    }
}

/// Emit the scalar rendering of `value`, or `None` when the value is not a
/// scalar.
fn render_scalar(value: &serde_yaml::Value) -> anyhow::Result<Option<String>> {
    use serde_yaml::Value as Y;
    Ok(match untag(value) {
        Y::Null => Some("None".to_string()),
        Y::Bool(b) => Some(b.to_string()),
        Y::Number(n) => Some(render_number(n)?),
        Y::String(s) => Some(serde_json::to_string(s).expect("string to JSON cannot fail")),
        _ => None,
    })
}

/// Write `level` 4-space indents.
fn write_indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str("    ");
    }
}

/// Emit a value at the given indent level. Scalars inline; arrays inline when
/// every element is a scalar and the rendering fits in 80 columns, otherwise
/// multiline with one element per indented line and trailing commas; mappings
/// always emit multiline (`{` / `key = value` lines / `}`), with `{}` when
/// empty.
fn emit_value(out: &mut String, value: &serde_yaml::Value, level: usize) -> anyhow::Result<()> {
    use serde_yaml::Value as Y;
    if let Some(scalar) = render_scalar(value)? {
        out.push_str(&scalar);
        return Ok(());
    }
    match untag(value) {
        Y::Sequence(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return Ok(());
            }
            // Try the inline form first: every element must be a scalar.
            let mut inline: Option<Vec<String>> = Some(Vec::with_capacity(items.len()));
            for item in items {
                match render_scalar(item)? {
                    Some(s) => inline.as_mut().expect("checked above").push(s),
                    None => {
                        inline = None;
                        break;
                    }
                }
            }
            if let Some(parts) = inline {
                let candidate = format!("[{}]", parts.join(", "));
                if candidate.len() <= 80 {
                    out.push_str(&candidate);
                    return Ok(());
                }
            }
            out.push_str("[\n");
            for item in items {
                write_indent(out, level + 1);
                emit_value(out, item, level + 1)?;
                out.push_str(",\n");
            }
            write_indent(out, level);
            out.push(']');
            Ok(())
        }
        Y::Mapping(map) => {
            if map.is_empty() {
                out.push_str("{}");
                return Ok(());
            }
            out.push_str("{\n");
            for (k, v) in map {
                let key = match untag(k) {
                    Y::String(s) => s.clone(),
                    other => {
                        anyhow::bail!("data object keys must be strings, found key {other:?}")
                    }
                };
                write_indent(out, level + 1);
                out.push_str(&render_key(&key));
                out.push_str(" = ");
                emit_value(out, v, level + 1)?;
                out.push('\n');
            }
            write_indent(out, level);
            out.push('}');
            Ok(())
        }
        other => anyhow::bail!("unsupported YAML value for KCL emission: {other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Convert a YAML document to TOML text through the same path as
    /// `GenerateToml`.
    fn to_toml(yaml: &str, sort_keys: bool) -> String {
        let value: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        toml::to_string(&yaml_to_toml(&value, sort_keys).unwrap()).unwrap()
    }

    /// Generate KCL from data content through the same path as
    /// `GenerateKcl`.
    fn to_kcl(format: DataFormat, source: &str) -> anyhow::Result<String> {
        value_to_kcl(&parse_data(format, source)?)
    }

    #[test]
    fn toml_preserves_source_order_by_default() {
        // `z` is a value, `a` is a table: TOML emits values before tables,
        // but group-internal order follows the source document.
        assert_eq!(
            to_toml("z: 1\na:\n  y: 2\n  b: 3\n", false),
            "z = 1\n\n[a]\ny = 2\nb = 3\n"
        );
    }

    #[test]
    fn toml_sort_keys_sorts_recursively() {
        assert_eq!(
            to_toml("z: 1\na:\n  y: 2\n  b: 3\n", true),
            "z = 1\n\n[a]\nb = 3\ny = 2\n"
        );
    }

    #[test]
    fn toml_moves_values_before_tables_within_a_table() {
        // Source order has the sub-table `a` before the value `b`; the TOML
        // serializer requires values first, so `b` moves up but nested
        // table order is otherwise preserved.
        assert_eq!(
            to_toml("a:\n  c: 1\nb: 2\n", false),
            "b = 2\n\n[a]\nc = 1\n"
        );
    }

    #[test]
    fn toml_drops_null_values() {
        assert_eq!(to_toml("a: null\nb: 1\n", false), "b = 1\n");
        // Nested nulls drop too, leaving the surrounding table intact.
        assert_eq!(to_toml("a:\n  x: null\n  y: 1\n", false), "[a]\ny = 1\n");
    }

    #[test]
    fn toml_rejects_null_array_elements() {
        let value: serde_yaml::Value = serde_yaml::from_str("- null\n").unwrap();
        let err = yaml_to_toml(&value, false).unwrap_err();
        assert!(
            err.to_string().contains("null array element"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn toml_converts_scalar_non_string_keys() {
        assert_eq!(
            to_toml("1: one\ntrue: t\n", false),
            "1 = \"one\"\ntrue = \"t\"\n"
        );
    }

    #[test]
    fn toml_rejects_complex_keys() {
        let value: serde_yaml::Value = serde_yaml::from_str("[1]: x\n").unwrap();
        let err = yaml_to_toml(&value, false).unwrap_err();
        assert!(
            err.to_string().contains("unsupported TOML key"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn format_resolution_precedence_and_validation() {
        assert_eq!(
            resolve_data_format("", "data.json").unwrap(),
            DataFormat::Json
        );
        assert_eq!(
            resolve_data_format("", "data.yaml").unwrap(),
            DataFormat::Yaml
        );
        assert_eq!(
            resolve_data_format("", "data.yml").unwrap(),
            DataFormat::Yaml
        );
        assert_eq!(
            resolve_data_format("", "data.toml").unwrap(),
            DataFormat::Toml
        );
        // Unknown or missing extensions default to JSON.
        assert_eq!(
            resolve_data_format("", "data.xml").unwrap(),
            DataFormat::Json
        );
        assert_eq!(resolve_data_format("", "noext").unwrap(), DataFormat::Json);
        // An explicit format wins over the extension and is case-insensitive.
        assert_eq!(
            resolve_data_format("JSON", "data.yaml").unwrap(),
            DataFormat::Json
        );
        assert_eq!(resolve_data_format("TOML", "").unwrap(), DataFormat::Toml);
        // An explicit but unknown format is an error.
        let err = resolve_data_format("xml", "data.json").unwrap_err();
        assert!(
            err.to_string().contains("unsupported data format"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn parse_rejects_empty_source() {
        let err = parse_data(DataFormat::Json, "  \n").unwrap_err();
        assert!(err.to_string().contains("source is empty"), "{err}");
    }

    #[test]
    fn kcl_scalars_and_float_rendering() {
        let kcl = to_kcl(
            DataFormat::Json,
            r#"{"i": 2, "f": 1.0, "half": 0.5, "t": true, "n": null, "s": "a\nb\"c"}"#,
        )
        .unwrap();
        assert_eq!(
            kcl,
            "i = 2\nf = 1.0\nhalf = 0.5\nt = true\nn = None\ns = \"a\\nb\\\"c\"\n"
        );
    }

    #[test]
    fn kcl_quotes_non_bare_keys() {
        let kcl = to_kcl(
            DataFormat::Json,
            r#"{"1a": 1, "a b": 2, "_ok": 3, "a\"b": 4}"#,
        )
        .unwrap();
        assert_eq!(kcl, "\"1a\" = 1\n\"a b\" = 2\n_ok = 3\n\"a\\\"b\" = 4\n");
    }

    #[test]
    fn kcl_nested_multiline_and_array_layouts() {
        let kcl = to_kcl(
            DataFormat::Json,
            r#"{"a": {"b": {"c": 1}, "d": [1, {"e": "x"}], "empty": [], "empty_obj": {}}}"#,
        )
        .unwrap();
        assert_eq!(
            kcl,
            "a = {\n    b = {\n        c = 1\n    }\n    d = [\n        1,\n        {\n            e = \"x\"\n        },\n    ]\n    empty = []\n    empty_obj = {}\n}\n"
        );
    }

    #[test]
    fn kcl_inlines_short_scalar_arrays_only() {
        assert_eq!(
            to_kcl(DataFormat::Json, r#"{"a": [1, 2, 3]}"#).unwrap(),
            "a = [1, 2, 3]\n"
        );
        // 30 two-digit numbers render to 117 chars inline — over the 80
        // column budget, so the array goes multiline.
        let source = format!(
            "{{\"long\": [{}]}}",
            (0..30).map(|_| "10").collect::<Vec<_>>().join(", ")
        );
        let kcl = to_kcl(DataFormat::Json, &source).unwrap();
        let expected = format!(
            "long = [\n{}]\n",
            (0..30).map(|_| "    10,\n").collect::<Vec<_>>().concat()
        );
        assert_eq!(kcl, expected);
    }

    #[test]
    fn kcl_root_must_be_an_object() {
        let err = to_kcl(DataFormat::Json, "[1, 2]").unwrap_err();
        assert!(
            err.to_string().contains("data root must be an object"),
            "unexpected error: {err}"
        );
        let err = to_kcl(DataFormat::Json, "42").unwrap_err();
        assert!(
            err.to_string().contains("data root must be an object"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn kcl_from_yaml_and_format_inference() {
        let fmt = resolve_data_format("", "data.yaml").unwrap();
        assert_eq!(fmt, DataFormat::Yaml);
        assert_eq!(
            to_kcl(fmt, "a:\n  b: 1\n").unwrap(),
            "a = {\n    b = 1\n}\n"
        );
    }

    #[test]
    fn kcl_from_toml_content() {
        let fmt = resolve_data_format("", "data.toml").unwrap();
        assert_eq!(fmt, DataFormat::Toml);
        // Ints and floats stay distinct through the TOML reader; datetimes
        // become plain strings.
        let kcl = to_kcl(fmt, "i = 42\nf = 1.0\nd = 1979-05-27T07:32:00Z\n").unwrap();
        assert_eq!(kcl, "i = 42\nf = 1.0\nd = \"1979-05-27T07:32:00Z\"\n");
    }

    #[test]
    fn kcl_explicit_format_wins_over_extension() {
        let fmt = resolve_data_format("json", "data.yaml").unwrap();
        assert_eq!(fmt, DataFormat::Json);
        assert_eq!(
            to_kcl(fmt, r#"{"a": {"b": 1}}"#).unwrap(),
            "a = {\n    b = 1\n}\n"
        );
    }
}
