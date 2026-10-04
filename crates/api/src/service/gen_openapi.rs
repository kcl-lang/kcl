//! Emitter backing the `GenerateOpenAPI` RPC.
//!
//! Converts the extracted package schema type mapping into an OpenAPI v3
//! (`openapi: 3.0.0` with `components/schemas`) or Swagger 2.0 document
//! (`swagger: 2.0` with `definitions`). Schema references go through the
//! shared name resolver, so cross-package collisions resolve identically to
//! the other emitters.

use std::collections::HashMap;

use crate::gpyrpc::{KclType, SchemaTypes};
use crate::service::gen_schema::{
    SchemaNameResolver, emit_schema_list, ordered_schemas, require_schemas,
};

/// How `$ref`s are spelled and whether `oneOf` is native, selected by the
/// target document flavor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RefStyle {
    /// OpenAPI v3: `#/components/schemas/Name`, native `oneOf`.
    OpenApiV3,
    /// Swagger 2.0: `#/definitions/Name`, unions use the `x-oneOf` vendor
    /// extension (Swagger 2.0 has no native `oneOf`).
    OpenApiV2,
    /// Standalone JSON Schema (used by the `GenerateDoc` "json-schema"
    /// format): refs by plain schema name, native `oneOf`.
    Plain,
}

impl RefStyle {
    fn ref_prefix(self) -> &'static str {
        match self {
            RefStyle::OpenApiV3 => "#/components/schemas/",
            RefStyle::OpenApiV2 => "#/definitions/",
            RefStyle::Plain => "",
        }
    }

    fn union_key(self) -> &'static str {
        match self {
            RefStyle::OpenApiV2 => "x-oneOf",
            RefStyle::OpenApiV3 | RefStyle::Plain => "oneOf",
        }
    }
}

/// Generate an OpenAPI/Swagger document from the schema mapping. `version`
/// is `"v3"` (also the default when empty) or `"v2"`.
pub(crate) fn generate_openapi(
    mapping: &HashMap<String, SchemaTypes>,
    version: &str,
) -> anyhow::Result<String> {
    let style = match version {
        "" | "v3" => RefStyle::OpenApiV3,
        "v2" => RefStyle::OpenApiV2,
        other => anyhow::bail!("unsupported OpenAPI version {other:?}; expected \"v3\" or \"v2\""),
    };
    let ordered = ordered_schemas(mapping);
    require_schemas(&ordered)?;
    let resolver = SchemaNameResolver::new(&ordered);
    let schemas = build_schema_definitions(&ordered, &resolver, style);
    let title = document_title(mapping);
    let mut root = serde_json::json!({
        "info": {"title": title, "version": "1.0.0"},
    });
    match style {
        RefStyle::OpenApiV3 => {
            root["openapi"] = serde_json::json!("3.0.0");
            root["components"] = serde_json::json!({"schemas": schemas});
        }
        RefStyle::OpenApiV2 => {
            root["swagger"] = serde_json::json!("2.0");
            root["definitions"] = serde_json::Value::Object(schemas);
        }
        RefStyle::Plain => unreachable!("Plain is not an OpenAPI document flavor"),
    }
    Ok(serde_json::to_string_pretty(&root)?)
}

/// Document title: the package name for a single-package mapping, otherwise
/// the generic "KCL Schemas".
fn document_title(mapping: &HashMap<String, SchemaTypes>) -> String {
    if mapping.len() == 1 {
        mapping.keys().next().expect("len checked").clone()
    } else {
        "KCL Schemas".to_string()
    }
}

/// Build the `components.schemas` / `definitions` object: one entry per
/// top-level schema plus the referenced-only closure.
pub(crate) fn build_schema_definitions(
    ordered: &[crate::service::gen_schema::PkgSchema<'_>],
    resolver: &SchemaNameResolver,
    style: RefStyle,
) -> serde_json::Map<String, serde_json::Value> {
    let list = emit_schema_list(ordered);
    let mut defs = serde_json::Map::new();
    for s in &list {
        defs.insert(s.name.clone(), schema_body(s.ty, resolver, style));
    }
    defs
}

/// Body of a top-level schema definition: an object schema with sorted
/// properties and the `required` list, wrapped in `allOf` against the base
/// schema reference when the schema has a base.
pub(crate) fn schema_body(
    ty: &KclType,
    resolver: &SchemaNameResolver,
    style: RefStyle,
) -> serde_json::Value {
    let mut body = serde_json::json!({"type": "object"});
    let properties = schema_properties(ty, resolver, style);
    if !properties.is_empty() {
        body["properties"] = serde_json::Value::Object(properties);
    }
    if !ty.required.is_empty() {
        body["required"] = serde_json::json!(ty.required);
    }
    if !ty.schema_doc.is_empty() {
        body["description"] = serde_json::json!(ty.schema_doc);
    }
    if let Some(base) = &ty.base_schema {
        let base_ref = schema_ref(base, resolver, style);
        return serde_json::json!({"allOf": [base_ref, body]});
    }
    body
}

/// Sorted (`BTreeMap`-ordered) properties object; function-typed fields are
/// skipped because neither JSON Schema nor proto can express them.
fn schema_properties(
    ty: &KclType,
    resolver: &SchemaNameResolver,
    style: RefStyle,
) -> serde_json::Map<String, serde_json::Value> {
    let mut keys: Vec<&String> = ty.properties.keys().collect();
    keys.sort();
    let mut properties = serde_json::Map::new();
    for k in keys {
        if let Some(v) = kcl_ty_to_json_schema(&ty.properties[k], resolver, style) {
            properties.insert(k.clone(), v);
        }
    }
    properties
}

fn schema_ref(ty: &KclType, resolver: &SchemaNameResolver, style: RefStyle) -> serde_json::Value {
    serde_json::json!({"$ref": format!("{}{}", style.ref_prefix(), resolver.name_of_ty(ty))})
}

/// Convert a (non-top-level) pb [`KclType`] into a JSON Schema fragment.
/// Function types yield `None` (the caller skips the field).
pub(crate) fn kcl_ty_to_json_schema(
    ty: &KclType,
    resolver: &SchemaNameResolver,
    style: RefStyle,
) -> Option<serde_json::Value> {
    let mut value = match ty.r#type.as_str() {
        "function" => return None,
        "str" => serde_json::json!({"type": "string"}),
        "int" => serde_json::json!({"type": "integer", "format": "int64"}),
        "float" | "number_multiplier" => serde_json::json!({"type": "number", "format": "double"}),
        "bool" => serde_json::json!({"type": "boolean"}),
        // Free-form value.
        "any" => serde_json::json!({}),
        "list" => {
            let items = ty
                .item
                .as_deref()
                .and_then(|item| kcl_ty_to_json_schema(item, resolver, style))
                .unwrap_or_else(|| serde_json::json!({}));
            serde_json::json!({"type": "array", "items": items})
        }
        "dict" => {
            // Only string keys are representable; the key type is ignored
            // (string keys assumed), matching proto `map<string, V>`.
            let additional = ty
                .item
                .as_deref()
                .and_then(|item| kcl_ty_to_json_schema(item, resolver, style))
                .unwrap_or_else(|| serde_json::json!({}));
            serde_json::json!({"type": "object", "additionalProperties": additional})
        }
        "schema" => schema_ref(ty, resolver, style),
        "union" => {
            let variants: Vec<serde_json::Value> = ty
                .union_types
                .iter()
                .filter_map(|u| kcl_ty_to_json_schema(u, resolver, style))
                .collect();
            serde_json::json!({style.union_key(): variants})
        }
        // Unknown types degrade to a free-form schema rather than failing
        // the whole document.
        _ => serde_json::json!({}),
    };
    if !ty.description.is_empty() {
        value["description"] = serde_json::json!(ty.description);
    }
    if !ty.default.is_empty() {
        value["default"] = serde_json::from_str(&ty.default)
            .unwrap_or_else(|_| serde_json::Value::String(ty.default.clone()));
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn int_ty() -> KclType {
        KclType {
            r#type: "int".to_string(),
            ..Default::default()
        }
    }

    fn str_ty() -> KclType {
        KclType {
            r#type: "str".to_string(),
            ..Default::default()
        }
    }

    fn schema_ref_ty(pkg: &str, name: &str) -> KclType {
        KclType {
            r#type: "schema".to_string(),
            schema_name: name.to_string(),
            pkg_path: pkg.to_string(),
            ..Default::default()
        }
    }

    fn single_schema_mapping(pkg: &str, ty: KclType) -> HashMap<String, SchemaTypes> {
        [(
            pkg.to_string(),
            SchemaTypes {
                schema_type: vec![ty],
            },
        )]
        .into_iter()
        .collect()
    }

    #[test]
    fn openapi_v3_golden_for_fixture() {
        let spec = generate_openapi(&fixture_mapping(), "v3").unwrap();
        assert_eq!(
            spec,
            r##"{
  "components": {
    "schemas": {
      "Address": {
        "description": "A postal address.",
        "properties": {
          "geo": {
            "additionalProperties": {
              "format": "double",
              "type": "number"
            },
            "description": "Arbitrary float coordinates keyed by name.",
            "type": "object"
          },
          "street": {
            "description": "The street name.",
            "type": "string"
          },
          "zipCode": {
            "description": "The ZIP code.",
            "type": "string"
          }
        },
        "required": [
          "street",
          "zipCode",
          "geo"
        ],
        "type": "object"
      },
      "Base": {
        "description": "Base schema carrying the shared identifier.",
        "properties": {
          "id": {
            "description": "The unique identifier.",
            "format": "int64",
            "type": "integer"
          }
        },
        "required": [
          "id"
        ],
        "type": "object"
      },
      "Person": {
        "allOf": [
          {
            "$ref": "#/components/schemas/Base"
          },
          {
            "description": "A person in the address book.",
            "properties": {
              "address": {
                "$ref": "#/components/schemas/Address",
                "description": "The home address."
              },
              "age": {
                "default": 18,
                "description": "The age in years.",
                "format": "int64",
                "type": "integer"
              },
              "emails": {
                "description": "Known email addresses.",
                "items": {
                  "type": "string"
                },
                "type": "array"
              },
              "id": {
                "description": "The unique identifier.",
                "format": "int64",
                "type": "integer"
              },
              "name": {
                "description": "The full name.",
                "type": "string"
              }
            },
            "required": [
              "name",
              "address",
              "id"
            ],
            "type": "object"
          }
        ]
      },
      "UnionS": {
        "description": "A schema with a union attribute.",
        "properties": {
          "val": {
            "description": "An int or a str.",
            "oneOf": [
              {
                "format": "int64",
                "type": "integer"
              },
              {
                "type": "string"
              }
            ]
          }
        },
        "required": [
          "val"
        ],
        "type": "object"
      }
    }
  },
  "info": {
    "title": "__main__",
    "version": "1.0.0"
  },
  "openapi": "3.0.0"
}"##
        );
    }

    #[test]
    fn openapi_v2_uses_definitions_and_x_oneof() {
        let spec = generate_openapi(&fixture_mapping(), "v2").unwrap();
        assert!(spec.contains("\"swagger\": \"2.0\""));
        assert!(spec.contains("\"definitions\": {"));
        assert!(spec.contains("\"$ref\": \"#/definitions/Address\""));
        assert!(spec.contains("\"x-oneOf\": ["));
        assert!(!spec.contains("\"oneOf\":"));
        // v2 refs must not leak v3 component paths.
        assert!(!spec.contains("#/components/schemas/"));
    }

    #[test]
    fn empty_version_defaults_to_v3() {
        let spec = generate_openapi(&fixture_mapping(), "").unwrap();
        assert!(spec.contains("\"openapi\": \"3.0.0\""));
    }

    #[test]
    fn unknown_version_is_an_error_naming_valid_values() {
        let err = generate_openapi(&fixture_mapping(), "v4").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("unsupported OpenAPI version"), "{msg}");
        assert!(msg.contains("\"v3\"") && msg.contains("\"v2\""), "{msg}");
    }

    #[test]
    fn empty_mapping_is_a_clear_error() {
        let err = generate_openapi(&HashMap::new(), "v3").unwrap_err();
        assert!(
            err.to_string().contains("no schemas found"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn union_renders_oneof_variants_in_order() {
        let mut ty = KclType {
            r#type: "schema".to_string(),
            schema_name: "U".to_string(),
            pkg_path: "main".to_string(),
            ..Default::default()
        };
        ty.properties.insert(
            "val".to_string(),
            KclType {
                r#type: "union".to_string(),
                union_types: vec![int_ty(), str_ty()],
                ..Default::default()
            },
        );
        ty.required.push("val".to_string());
        let mapping = single_schema_mapping("main", ty);
        let ordered = ordered_schemas(&mapping);
        let resolver = SchemaNameResolver::new(&ordered);
        let body = schema_body(
            &mapping["main"].schema_type[0],
            &resolver,
            RefStyle::OpenApiV3,
        );
        assert_eq!(
            body["properties"]["val"]["oneOf"],
            serde_json::json!([
                {"type": "integer", "format": "int64"},
                {"type": "string"}
            ])
        );
    }

    #[test]
    fn base_schema_renders_allof_with_ref() {
        let mut ty = KclType {
            r#type: "schema".to_string(),
            schema_name: "Derived".to_string(),
            pkg_path: "main".to_string(),
            base_schema: Some(Box::new(schema_ref_ty("main", "Base"))),
            ..Default::default()
        };
        ty.properties.insert("own".to_string(), str_ty());
        let mapping = single_schema_mapping("main", ty);
        let ordered = ordered_schemas(&mapping);
        let resolver = SchemaNameResolver::new(&ordered);
        let body = schema_body(
            &mapping["main"].schema_type[0],
            &resolver,
            RefStyle::OpenApiV3,
        );
        assert_eq!(
            body["allOf"][0]["$ref"],
            serde_json::json!("#/components/schemas/Base")
        );
        assert_eq!(
            body["allOf"][1]["properties"]["own"],
            serde_json::json!({"type": "string"})
        );
    }

    #[test]
    fn dict_uses_additional_properties_and_ignores_key_type() {
        let mut ty = KclType {
            r#type: "schema".to_string(),
            schema_name: "D".to_string(),
            pkg_path: "main".to_string(),
            ..Default::default()
        };
        ty.properties.insert(
            "m".to_string(),
            KclType {
                r#type: "dict".to_string(),
                key: Some(Box::new(int_ty())),
                item: Some(Box::new(str_ty())),
                ..Default::default()
            },
        );
        let mapping = single_schema_mapping("main", ty);
        let ordered = ordered_schemas(&mapping);
        let resolver = SchemaNameResolver::new(&ordered);
        let body = schema_body(
            &mapping["main"].schema_type[0],
            &resolver,
            RefStyle::OpenApiV3,
        );
        assert_eq!(
            body["properties"]["m"],
            serde_json::json!({
                "type": "object",
                "additionalProperties": {"type": "string"}
            })
        );
    }

    #[test]
    fn name_collisions_across_packages_get_suffixed_refs() {
        let mut a = schema_ref_ty("aaa", "Item");
        a.properties.insert("v".to_string(), int_ty());
        let mut b = schema_ref_ty("bbb", "Item");
        b.properties.insert("v".to_string(), int_ty());
        let mapping: HashMap<String, SchemaTypes> = [
            (
                "aaa".to_string(),
                SchemaTypes {
                    schema_type: vec![a],
                },
            ),
            (
                "bbb".to_string(),
                SchemaTypes {
                    schema_type: vec![b],
                },
            ),
        ]
        .into_iter()
        .collect();
        let spec = generate_openapi(&mapping, "v3").unwrap();
        // Colliding schema names become unique definition keys (nothing
        // references them here, so no $ref strings are emitted).
        assert!(spec.contains("\"Item_aaa\": {"));
        assert!(spec.contains("\"Item_bbb\": {"));
        assert!(!spec.contains("\"Item\": {"));
        // Multi-package document uses the generic title.
        assert!(spec.contains("\"title\": \"KCL Schemas\""));
    }

    #[test]
    fn required_list_is_preserved_in_order() {
        let mut ty = KclType {
            r#type: "schema".to_string(),
            schema_name: "R".to_string(),
            pkg_path: "main".to_string(),
            ..Default::default()
        };
        ty.properties.insert("a".to_string(), int_ty());
        ty.properties.insert("b".to_string(), str_ty());
        ty.required = vec!["b".to_string(), "a".to_string()];
        let mapping = single_schema_mapping("main", ty);
        let ordered = ordered_schemas(&mapping);
        let resolver = SchemaNameResolver::new(&ordered);
        let body = schema_body(
            &mapping["main"].schema_type[0],
            &resolver,
            RefStyle::OpenApiV3,
        );
        assert_eq!(body["required"], serde_json::json!(["b", "a"]));
    }

    #[test]
    fn function_fields_are_skipped() {
        let mut ty = KclType {
            r#type: "schema".to_string(),
            schema_name: "F".to_string(),
            pkg_path: "main".to_string(),
            ..Default::default()
        };
        ty.properties.insert(
            "cb".to_string(),
            KclType {
                r#type: "function".to_string(),
                ..Default::default()
            },
        );
        ty.properties.insert("v".to_string(), int_ty());
        let mapping = single_schema_mapping("main", ty);
        let ordered = ordered_schemas(&mapping);
        let resolver = SchemaNameResolver::new(&ordered);
        let body = schema_body(
            &mapping["main"].schema_type[0],
            &resolver,
            RefStyle::OpenApiV3,
        );
        assert!(body["properties"].get("cb").is_none());
        assert!(body["properties"].get("v").is_some());
    }
}
