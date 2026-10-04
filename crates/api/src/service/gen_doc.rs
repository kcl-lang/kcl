//! Emitter backing the `GenerateDoc` RPC.
//!
//! Formats the extracted package schema type mapping as documentation:
//!
//! * `"md"` (default) — a Markdown document with one section per package and
//!   an attributes table per schema;
//! * `"openapi"` — the Swagger 2.0 document produced by the shared
//!   [`crate::service::gen_openapi`] v2 emitter;
//! * `"json-schema"` — a standalone `{schemaName: <JSON Schema>}` object
//!   built by the same KclType conversion as the OpenAPI v3 bodies, with
//!   `$ref`s spelled by plain schema name.
//!
//! `"html"` is reserved but not supported yet and fails with an explicit
//! error.

use std::collections::HashMap;

use crate::gpyrpc::SchemaTypes;
use crate::service::gen_openapi::{self, RefStyle};
use crate::service::gen_schema::{
    SchemaNameResolver, emit_schema_list, kcl_ty_type_string, ordered_schemas, require_schemas,
};

/// Generate documentation from the schema mapping. `format` is `"md"`
/// (default when empty), `"openapi"`, `"json-schema"` or `"html"`.
pub(crate) fn generate_doc(
    mapping: &HashMap<String, SchemaTypes>,
    format: &str,
) -> anyhow::Result<String> {
    match format {
        "" | "md" => generate_markdown(mapping),
        "openapi" => gen_openapi::generate_openapi(mapping, "v2"),
        "json-schema" => generate_json_schema(mapping),
        "html" => anyhow::bail!("the \"html\" doc format is not supported yet"),
        other => anyhow::bail!(
            "unsupported doc format {other:?}; expected one of \"md\", \"openapi\" or \"json-schema\""
        ),
    }
}

/// Markdown document: `# Schemas` title, one `## Package <pkg>` section per
/// package (sorted), one `### <Schema>` subsection per top-level schema with
/// its doc paragraph and a deterministic attributes table (rows sorted by
/// name). Referenced-only closure schemas are not documented.
fn generate_markdown(mapping: &HashMap<String, SchemaTypes>) -> anyhow::Result<String> {
    let ordered = ordered_schemas(mapping);
    require_schemas(&ordered)?;
    let list = emit_schema_list(&ordered);
    let mut out = String::from("# Schemas\n");
    let mut current_pkg: Option<&str> = None;
    for s in &list {
        // The closure is appended after every top-level schema.
        if !s.top_level {
            break;
        }
        if current_pkg != Some(s.ty.pkg_path.as_str()) {
            out.push_str(&format!("\n## Package {}\n", s.ty.pkg_path));
            current_pkg = Some(s.ty.pkg_path.as_str());
        }
        out.push_str(&format!("\n### {}\n\n", s.ty.schema_name));
        if !s.ty.schema_doc.is_empty() {
            out.push_str(s.ty.schema_doc.trim());
            out.push_str("\n\n");
        }
        out.push_str("| Name | Type | Required | Default | Description |\n");
        out.push_str("| --- | --- | --- | --- | --- |\n");
        let mut keys: Vec<&String> = s.ty.properties.keys().collect();
        keys.sort();
        for k in keys {
            let prop = &s.ty.properties[k];
            let required = if s.ty.required.iter().any(|r| r == k) {
                "yes"
            } else {
                "no"
            };
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                escape_table_cell(k),
                escape_table_cell(&kcl_ty_type_string(prop)),
                required,
                escape_table_cell(&prop.default),
                escape_table_cell(&prop.description),
            ));
        }
    }
    Ok(out)
}

/// Escape the Markdown table cell separators and newlines of raw text.
fn escape_table_cell(text: &str) -> String {
    text.replace('|', "\\|").replace('\n', " ")
}

/// Standalone JSON Schema document: one draft-07-ish definition per schema,
/// `$ref`s by plain schema name.
fn generate_json_schema(mapping: &HashMap<String, SchemaTypes>) -> anyhow::Result<String> {
    let ordered = ordered_schemas(mapping);
    require_schemas(&ordered)?;
    let resolver = SchemaNameResolver::new(&ordered);
    let list = emit_schema_list(&ordered);
    let mut defs = serde_json::Map::new();
    for s in &list {
        defs.insert(
            s.name.clone(),
            gen_openapi::schema_body(s.ty, &resolver, RefStyle::Plain),
        );
    }
    Ok(serde_json::to_string_pretty(&serde_json::Value::Object(
        defs,
    ))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpyrpc::KclType;
    use crate::service::gen_schema::load_pkg_schema_types;
    use std::path::Path;

    fn fixture_mapping() -> HashMap<String, SchemaTypes> {
        let path = Path::new(".")
            .join("src")
            .join("testdata")
            .join("gen_openapi")
            .join("main.k")
            .canonicalize()
            .unwrap()
            .display()
            .to_string();
        load_pkg_schema_types(
            &crate::gpyrpc::ParseProgramArgs {
                paths: vec![path],
                ..Default::default()
            },
            "",
        )
        .unwrap()
    }

    #[test]
    fn markdown_golden_for_fixture() {
        let md = generate_doc(&fixture_mapping(), "md").unwrap();
        assert_eq!(
            md,
            r#"# Schemas

## Package __main__

### Base

Base schema carrying the shared identifier.

| Name | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| id | int | yes |  | The unique identifier. |

### Address

A postal address.

| Name | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| geo | {str:float} | yes |  | Arbitrary float coordinates keyed by name. |
| street | str | yes |  | The street name. |
| zipCode | str | yes |  | The ZIP code. |

### Person

A person in the address book.

| Name | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| address | Address | yes |  | The home address. |
| age | int | no | 18 | The age in years. |
| emails | [str] | no |  | Known email addresses. |
| id | int | yes |  | The unique identifier. |
| name | str | yes |  | The full name. |

### UnionS

A schema with a union attribute.

| Name | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| val | int \| str | yes |  | An int or a str. |
"#
        );
    }

    #[test]
    fn empty_format_defaults_to_markdown() {
        let md = generate_doc(&fixture_mapping(), "").unwrap();
        assert!(md.starts_with("# Schemas\n"));
    }

    #[test]
    fn openapi_format_reuses_the_v2_emitter() {
        let doc = generate_doc(&fixture_mapping(), "openapi").unwrap();
        assert!(doc.contains("\"swagger\": \"2.0\""));
        assert!(doc.contains("\"definitions\": {"));
        // Person carries a base-schema ref into the v2 definitions space.
        assert!(doc.contains("\"$ref\": \"#/definitions/Base\""));
    }

    #[test]
    fn json_schema_format_uses_plain_name_refs() {
        let doc = generate_doc(&fixture_mapping(), "json-schema").unwrap();
        let value: serde_json::Value = serde_json::from_str(&doc).unwrap();
        assert!(value.get("Person").is_some());
        assert!(value.get("Address").is_some());
        assert!(value.get("Base").is_some());
        assert!(value.get("UnionS").is_some());
        // Base-schema refs resolve by plain schema name.
        assert_eq!(
            value["Person"]["allOf"][0]["$ref"],
            serde_json::json!("Base")
        );
        // Union members keep the native oneOf key outside Swagger.
        assert!(value["UnionS"]["properties"]["val"]["oneOf"].is_array());
    }

    #[test]
    fn html_format_is_an_explicit_error() {
        let err = generate_doc(&fixture_mapping(), "html").unwrap_err();
        assert!(
            err.to_string().contains("not supported yet"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn unknown_format_is_an_error_naming_valid_values() {
        let err = generate_doc(&fixture_mapping(), "rst").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("unsupported doc format"), "{msg}");
        for valid in ["md", "openapi", "json-schema"] {
            assert!(msg.contains(valid), "{msg}");
        }
    }

    #[test]
    fn empty_mapping_is_a_clear_error() {
        let err = generate_doc(&HashMap::new(), "md").unwrap_err();
        assert!(
            err.to_string().contains("no schemas found"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn table_cells_escape_pipes_and_newlines() {
        let mut ty = KclType {
            r#type: "schema".to_string(),
            schema_name: "T".to_string(),
            pkg_path: "main".to_string(),
            ..Default::default()
        };
        let mut prop = KclType {
            r#type: "str".to_string(),
            description: "a | b\nc".to_string(),
            ..Default::default()
        };
        prop.default = "x|y".to_string();
        ty.properties.insert("v".to_string(), prop);
        ty.required.push("v".to_string());
        let mapping: HashMap<String, SchemaTypes> = [(
            "main".to_string(),
            SchemaTypes {
                schema_type: vec![ty],
            },
        )]
        .into_iter()
        .collect();
        let md = generate_doc(&mapping, "md").unwrap();
        assert!(md.contains("| v | str | yes | x\\|y | a \\| b c |"), "{md}");
    }
}
