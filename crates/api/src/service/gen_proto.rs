//! Emitter backing the `GenerateProto` RPC.
//!
//! Converts the extracted package schema type mapping into proto3 message
//! definitions. Notes on the mapping choices:
//!
//! * KCL attribute names are `snake_case`d (proto field naming convention);
//!   collisions after snake-casing (e.g. `fooBar` and `foo_bar`) are resolved
//!   with a deterministic numeric suffix (`foo_bar`, `foo_bar_2`).
//! * Union and `any` fields use `google.protobuf.Value`; the
//!   `google/protobuf/struct.proto` import is emitted only when at least one
//!   field needs it.
//! * KCL base-schema inheritance has no proto equivalent: attributes are
//!   already flattened into the derived message by the extractor, and the
//!   base schema itself is emitted as its own message (via the shared
//!   referenced-only closure).
//! * Index signatures cannot be expressed as proto fields without inventing
//!   a field name; they are rendered as a `// index signature:` comment.
//! * Function-typed fields are skipped (proto cannot express them).

use std::collections::HashMap;

use crate::gpyrpc::{KclType, SchemaTypes};
use crate::service::gen_schema::{
    SchemaNameResolver, emit_schema_list, kcl_ty_type_string, ordered_schemas, require_schemas,
};

/// Well-known type used for union and `any` fields.
const VALUE_TYPE: &str = "google.protobuf.Value";
const VALUE_IMPORT: &str = "google/protobuf/struct.proto";

/// Generate proto3 definitions from the schema mapping. `package` is the
/// proto package name; empty means no `package` clause.
pub(crate) fn generate_proto(
    mapping: &HashMap<String, SchemaTypes>,
    package: &str,
) -> anyhow::Result<String> {
    let ordered = ordered_schemas(mapping);
    require_schemas(&ordered)?;
    let resolver = SchemaNameResolver::new(&ordered);
    let list = emit_schema_list(&ordered);

    let mut out = String::new();
    out.push_str("syntax = \"proto3\";\n");
    out.push('\n');
    if !package.is_empty() {
        out.push_str(&format!("package {package};\n\n"));
    }
    // Messages are rendered first to know whether the Value import is
    // needed; the header is already emitted, so buffer the bodies.
    let mut body = String::new();
    let mut uses_value = false;
    for s in &list {
        let msg = emit_message(&s.name, s.ty, &resolver)?;
        uses_value |= msg.uses_value;
        body.push_str(&msg.text);
        body.push('\n');
    }
    if uses_value {
        out.push_str(&format!("import \"{VALUE_IMPORT}\";\n\n"));
    }
    out.push_str(&body);
    Ok(out)
}

struct MessageOut {
    text: String,
    uses_value: bool,
}

fn emit_message(
    name: &str,
    ty: &KclType,
    resolver: &SchemaNameResolver,
) -> anyhow::Result<MessageOut> {
    let mut text = String::new();
    for line in ty.schema_doc.lines() {
        if line.trim().is_empty() {
            text.push_str("//\n");
        } else {
            text.push_str(&format!("// {line}\n"));
        }
    }
    text.push_str(&format!("message {name} {{\n"));
    let mut uses_value = false;
    // Fields in sorted property order so numbering is deterministic.
    let mut keys: Vec<&String> = ty.properties.keys().collect();
    keys.sort();
    let mut used_names: Vec<String> = Vec::new();
    let mut field_no = 1u32;
    for k in keys {
        let prop = &ty.properties[k];
        if prop.r#type == "function" {
            // proto3 cannot express function-typed fields; skip them.
            text.push_str(&format!("    // skipped function field {k}\n"));
            continue;
        }
        let (ftype, field_uses_value) = field_type(prop, resolver, k)?;
        uses_value |= field_uses_value;
        let fname = unique_snake_case(k, &mut used_names);
        if !prop.description.is_empty() {
            for line in prop.description.lines() {
                text.push_str(&format!("    // {line}\n"));
            }
        }
        text.push_str(&format!("    {ftype} {fname} = {field_no};\n"));
        field_no += 1;
    }
    if let Some(sig) = &ty.index_signature {
        // Index signatures are open-ended maps keyed by string; rendered as
        // a comment because proto has no anonymous map field.
        let val = sig
            .val
            .as_deref()
            .map(kcl_ty_type_string)
            .unwrap_or_default();
        text.push_str(&format!("    // index signature: map<string, {val}>\n"));
    }
    text.push_str("}\n");
    Ok(MessageOut { text, uses_value })
}

/// Resolve the proto type of a field. Returns the type spelling and whether
/// it uses `google.protobuf.Value`.
fn field_type(
    ty: &KclType,
    resolver: &SchemaNameResolver,
    field: &str,
) -> anyhow::Result<(String, bool)> {
    Ok(match ty.r#type.as_str() {
        "str" => ("string".to_string(), false),
        "int" | "number_multiplier" => ("int64".to_string(), false),
        "float" => ("double".to_string(), false),
        "bool" => ("bool".to_string(), false),
        "any" | "union" => (VALUE_TYPE.to_string(), true),
        "schema" => (resolver.name_of_ty(ty), false),
        "list" => {
            let item = ty
                .item
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("list field {field:?} has no item type"))?;
            let (item_ty, uses_value) = field_type(item, resolver, field)?;
            if item_ty.starts_with("map<") || item_ty.starts_with("repeated ") {
                anyhow::bail!(
                    "list field {field:?} has item type {:?} that proto cannot express as a repeated element",
                    item.r#type
                );
            }
            (format!("repeated {item_ty}"), uses_value)
        }
        "dict" => {
            let key = ty
                .key
                .as_deref()
                .map(|k| k.r#type.as_str())
                .unwrap_or("str");
            if key != "str" {
                anyhow::bail!(
                    "dict field {field:?} has non-string key type {key:?}; proto map keys must be strings"
                );
            }
            let item = ty
                .item
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("dict field {field:?} has no value type"))?;
            let (item_ty, uses_value) = field_type(item, resolver, field)?;
            if item_ty.starts_with("map<") || item_ty.starts_with("repeated ") {
                anyhow::bail!(
                    "dict field {field:?} has value type {:?} that proto cannot express as a map value",
                    item.r#type
                );
            }
            (format!("map<string, {item_ty}>"), uses_value)
        }
        other => anyhow::bail!("unsupported KCL type {other:?} for proto field {field:?}"),
    })
}

/// Convert a KCL attribute name to `snake_case`: an uppercase letter starts
/// a new word (underscore prefix) when the previous character is lowercase
/// or a digit, or when it ends an acronym (previous uppercase followed by a
/// lowercase letter). Characters outside `[A-Za-z0-9_]` become underscores.
fn to_snake_case(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::new();
    for (i, c) in chars.iter().enumerate() {
        if c.is_ascii_uppercase() {
            let prev = i.checked_sub(1).map(|p| chars[p]);
            let next = chars.get(i + 1);
            let word_boundary = matches!(prev, Some(p) if p.is_ascii_lowercase() || p.is_ascii_digit())
                || (matches!(prev, Some(p) if p.is_ascii_uppercase())
                    && matches!(next, Some(n) if n.is_ascii_lowercase()));
            if word_boundary && !out.ends_with('_') {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else if c.is_ascii_alphanumeric() || *c == '_' {
            out.push(*c);
        } else {
            out.push('_');
        }
    }
    out
}

/// Reserve `name` in `used`, appending `_2`, `_3`, ... on collision so that
/// distinct KCL attributes that snake_case to the same proto name still get
/// distinct, deterministic field names.
fn unique_snake_case(name: &str, used: &mut Vec<String>) -> String {
    let base = to_snake_case(name);
    if !used.contains(&base) {
        used.push(base.clone());
        return base;
    }
    let mut n = 2;
    loop {
        let candidate = format!("{base}_{n}");
        if !used.contains(&candidate) {
            used.push(candidate.clone());
            return candidate;
        }
        n += 1;
    }
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
    fn proto_golden_for_fixture_with_package() {
        let proto = generate_proto(&fixture_mapping(), "example.v1").unwrap();
        assert_eq!(
            proto,
            r#"syntax = "proto3";

package example.v1;

import "google/protobuf/struct.proto";

// Base schema carrying the shared identifier.
message Base {
    // The unique identifier.
    int64 id = 1;
}

// A postal address.
message Address {
    // Arbitrary float coordinates keyed by name.
    map<string, double> geo = 1;
    // The street name.
    string street = 2;
    // The ZIP code.
    string zip_code = 3;
}

// A person in the address book.
message Person {
    // The home address.
    Address address = 1;
    // The age in years.
    int64 age = 2;
    // Known email addresses.
    repeated string emails = 3;
    // The unique identifier.
    int64 id = 4;
    // The full name.
    string name = 5;
}

// A schema with a union attribute.
message UnionS {
    // An int or a str.
    google.protobuf.Value val = 1;
}

"#
        );
    }

    #[test]
    fn proto_without_package_omits_package_clause_and_value_import() {
        let mut ty = KclType {
            r#type: "schema".to_string(),
            schema_name: "Only".to_string(),
            pkg_path: "main".to_string(),
            ..Default::default()
        };
        ty.properties.insert("a".to_string(), int_ty());
        let mapping = single_schema_mapping("main", ty);
        let proto = generate_proto(&mapping, "").unwrap();
        assert_eq!(
            proto,
            "syntax = \"proto3\";\n\nmessage Only {\n    int64 a = 1;\n}\n\n"
        );
        assert!(!proto.contains("package "));
        assert!(!proto.contains("struct.proto"));
    }

    #[test]
    fn snake_case_conversion_and_collisions() {
        assert_eq!(to_snake_case("name"), "name");
        assert_eq!(to_snake_case("zipCode"), "zip_code");
        assert_eq!(to_snake_case("ZIPCode"), "zip_code");
        assert_eq!(to_snake_case("a b"), "a_b");
        let mut used = Vec::new();
        assert_eq!(unique_snake_case("fooBar", &mut used), "foo_bar");
        assert_eq!(unique_snake_case("foo_bar", &mut used), "foo_bar_2");
        assert_eq!(unique_snake_case("foo_bar", &mut used), "foo_bar_3");
    }

    #[test]
    fn dict_with_non_string_key_is_an_error_naming_the_field() {
        let mut ty = KclType {
            r#type: "schema".to_string(),
            schema_name: "D".to_string(),
            pkg_path: "main".to_string(),
            ..Default::default()
        };
        ty.properties.insert(
            "weird".to_string(),
            KclType {
                r#type: "dict".to_string(),
                key: Some(Box::new(int_ty())),
                item: Some(Box::new(str_ty())),
                ..Default::default()
            },
        );
        let mapping = single_schema_mapping("main", ty);
        let err = generate_proto(&mapping, "").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("non-string key type"), "{msg}");
        assert!(msg.contains("weird"), "{msg}");
    }

    #[test]
    fn union_and_any_fields_use_value_with_conditional_import() {
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
        ty.properties.insert(
            "free".to_string(),
            KclType {
                r#type: "any".to_string(),
                ..Default::default()
            },
        );
        let mapping = single_schema_mapping("main", ty);
        let proto = generate_proto(&mapping, "").unwrap();
        assert!(proto.contains("import \"google/protobuf/struct.proto\";"));
        // Fields are numbered in sorted property order: free < val.
        assert!(proto.contains("google.protobuf.Value free = 1;"));
        assert!(proto.contains("google.protobuf.Value val = 2;"));
    }

    #[test]
    fn referenced_only_base_schema_is_emitted_as_closure_message() {
        let mut external = schema_ref_ty("ext", "External");
        external.properties.insert("x".to_string(), int_ty());
        let mut derived = schema_ref_ty("main", "Derived");
        derived.base_schema = Some(Box::new(external));
        derived.properties.insert("own".to_string(), str_ty());
        let mapping = single_schema_mapping("main", derived);
        let proto = generate_proto(&mapping, "").unwrap();
        assert!(proto.contains("message Derived {"));
        assert!(proto.contains("message External {"));
        // The closure message carries the base attribute.
        assert!(proto.contains("int64 x = 1;"));
        // Derived keeps only its own attribute.
        assert!(proto.contains("string own = 1;"));
    }

    #[test]
    fn function_fields_are_skipped_with_a_comment() {
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
        let proto = generate_proto(&mapping, "").unwrap();
        assert!(proto.contains("// skipped function field cb"));
        assert!(proto.contains("int64 v = 1;"));
        assert!(!proto.contains("cb ="));
    }

    #[test]
    fn index_signature_is_rendered_as_a_comment() {
        let mut ty = KclType {
            r#type: "schema".to_string(),
            schema_name: "I".to_string(),
            pkg_path: "main".to_string(),
            ..Default::default()
        };
        ty.index_signature = Some(Box::new(crate::gpyrpc::IndexSignature {
            key_name: Some("key".to_string()),
            key: Some(Box::new(str_ty())),
            val: Some(Box::new(int_ty())),
            any_other: false,
        }));
        let mapping = single_schema_mapping("main", ty);
        let proto = generate_proto(&mapping, "").unwrap();
        assert!(proto.contains("// index signature: map<string, int>"));
    }

    #[test]
    fn empty_mapping_is_a_clear_error() {
        let err = generate_proto(&HashMap::new(), "").unwrap_err();
        assert!(
            err.to_string().contains("no schemas found"),
            "unexpected error: {err}"
        );
    }
}
