//! Shared schema extraction for the schema-driven generator RPCs
//! (`GenerateOpenAPI`, `GenerateProto`, `GenerateDoc`).
//!
//! All three emitters consume the same extracted model: the package schema
//! type mapping produced by [`load_pkg_schema_types`], plus the deterministic
//! ordering ([`ordered_schemas`]) and cross-package name collision resolution
//! ([`SchemaNameResolver`]) shared by every emitter so that references to the
//! same schema render identically no matter the target format.

use std::collections::HashMap;

use crate::gpyrpc::{self, KclType, ParseProgramArgs, SchemaTypes};
use crate::service::ty::kcl_schema_ty_to_pb_ty;
use kcl_parser::LoadProgramOptions;
use kcl_query::GetSchemaOption;
use kcl_query::query::{CompilationOptions, get_full_schema_type_under_path};
use kcl_sema::resolver::Options;

/// Load the program referenced by `parse_args` and collect every schema
/// definition, grouped by package name (main package, external packages and
/// sub-packages).
///
/// This is the shared load-and-collect path previously inlined in
/// `get_schema_type_mapping_under_path`; that RPC now delegates here.
/// `work_dir` resolves relative entry paths; the generation RPCs pass ""
/// (their `ParseProgramArgs` has no working-directory knob), while
/// `GetSchemaTypeMappingUnderPath` forwards `exec_args.work_dir` to preserve
/// its pre-refactor behavior.
pub(crate) fn load_pkg_schema_types(
    parse_args: &ParseProgramArgs,
    work_dir: &str,
) -> anyhow::Result<HashMap<String, gpyrpc::SchemaTypes>> {
    let mut package_maps = HashMap::new();
    for p in &parse_args.external_pkgs {
        package_maps.insert(p.pkg_name.clone(), p.pkg_path.clone());
    }
    let mut type_mapping = HashMap::new();
    for (pkg, schema_tys) in get_full_schema_type_under_path(
        None,
        CompilationOptions {
            paths: parse_args.paths.clone(),
            loader_opts: Some(LoadProgramOptions {
                k_code_list: parse_args.sources.clone(),
                package_maps,
                load_plugins: true,
                work_dir: work_dir.to_string(),
                ..Default::default()
            }),
            resolve_opts: Options {
                resolve_val: true,
                ..Default::default()
            },
            get_schema_opts: GetSchemaOption::Definitions,
        },
    )? {
        let mut tys = Vec::with_capacity(schema_tys.len());
        for schema_ty in &schema_tys {
            tys.push(kcl_schema_ty_to_pb_ty(schema_ty));
        }
        type_mapping.insert(pkg, gpyrpc::SchemaTypes { schema_type: tys });
    }
    Ok(type_mapping)
}

/// A schema definition paired with the name of the package it belongs to.
pub(crate) struct PkgSchema<'a> {
    pub pkg: &'a str,
    pub ty: &'a KclType,
}

/// All schemas of the mapping in a deterministic global order: packages
/// sorted by name, schemas within a package in mapping list order (the order
/// the loader collected them).
pub(crate) fn ordered_schemas<'a>(mapping: &'a HashMap<String, SchemaTypes>) -> Vec<PkgSchema<'a>> {
    let mut pkgs: Vec<&String> = mapping.keys().collect();
    pkgs.sort();
    let mut result = Vec::new();
    for pkg in pkgs {
        for ty in &mapping[pkg].schema_type {
            result.push(PkgSchema {
                pkg: pkg.as_str(),
                ty,
            });
        }
    }
    result
}

/// Bail out with a clear message when the parsed program defines no schemas.
pub(crate) fn require_schemas(ordered: &[PkgSchema<'_>]) -> anyhow::Result<()> {
    if ordered.is_empty() {
        anyhow::bail!("no schemas found in the parsed program");
    }
    Ok(())
}

/// Deterministic unique names for every schema in the mapping. Schemas from
/// different packages keep their own name unless the bare schema name
/// collides across packages; collisions are resolved by suffixing the
/// sanitized package name (`Name_pkg`), keeping output stable.
pub(crate) struct SchemaNameResolver {
    /// (package name, schema name) -> emitted name.
    names: HashMap<(String, String), String>,
}

impl SchemaNameResolver {
    pub fn new(ordered: &[PkgSchema<'_>]) -> Self {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for s in ordered {
            *counts.entry(s.ty.schema_name.as_str()).or_default() += 1;
        }
        let mut names = HashMap::new();
        for s in ordered {
            let emitted = if counts[s.ty.schema_name.as_str()] > 1 {
                format!("{}_{}", s.ty.schema_name, sanitize_pkg_name(s.pkg))
            } else {
                s.ty.schema_name.clone()
            };
            names.insert((s.pkg.to_string(), s.ty.schema_name.clone()), emitted);
        }
        Self { names }
    }

    /// Emitted name for a schema reference. Targets that are not part of the
    /// mapping (e.g. an external base schema that was never loaded) fall back
    /// to their bare schema name.
    pub fn name(&self, pkg: &str, schema_name: &str) -> String {
        self.names
            .get(&(pkg.to_string(), schema_name.to_string()))
            .cloned()
            .unwrap_or_else(|| schema_name.to_string())
    }

    /// Emitted name for a schema-typed [`KclType`] reference.
    pub fn name_of_ty(&self, ty: &KclType) -> String {
        self.name(&ty.pkg_path, &ty.schema_name)
    }
}

/// Replace characters that are not valid in an identifier-ish suffix.
fn sanitize_pkg_name(pkg: &str) -> String {
    pkg.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Append the schemas referenced by `ordered` (through `base_schema` links or
/// One entry of the emission list: a schema under its resolved unique name.
pub(crate) struct EmittedSchema<'a> {
    /// Resolved (unique) name of the schema.
    pub name: String,
    /// `true` for top-level schemas, `false` for closure (referenced-only)
    /// schemas.
    pub top_level: bool,
    pub ty: &'a KclType,
}

/// The emission list: every top-level schema (deterministic
/// [`ordered_schemas`] order) followed by the referenced-only closure —
/// schemas reachable through `base_schema` links or schema-typed properties
/// that are not themselves top-level entries of the mapping, so every
/// emitted reference resolves. The closure is computed breadth-first in
/// traversal order, and referenced schemas render under their resolved name.
pub(crate) fn emit_schema_list<'a>(ordered: &'a [PkgSchema<'a>]) -> Vec<EmittedSchema<'a>> {
    let resolver = SchemaNameResolver::new(ordered);
    let mut result: Vec<EmittedSchema<'_>> = Vec::new();
    let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
    for s in ordered {
        seen.insert((s.pkg.to_string(), s.ty.schema_name.clone()));
        result.push(EmittedSchema {
            name: resolver.name(s.pkg, &s.ty.schema_name),
            top_level: true,
            ty: s.ty,
        });
    }
    // Breadth-first: `i` walks `result`, appending newly discovered
    // referenced-only schemas; each is visited once via `seen`.
    let mut i = 0;
    while i < result.len() {
        let ty = result[i].ty;
        i += 1;
        let mut refs: Vec<&KclType> = Vec::new();
        if let Some(base) = &ty.base_schema {
            refs.push(base);
        }
        collect_schema_refs(ty, &mut refs);
        for r in refs {
            let key = (r.pkg_path.clone(), r.schema_name.clone());
            if r.r#type == "schema" && !r.schema_name.is_empty() && seen.insert(key) {
                result.push(EmittedSchema {
                    name: resolver.name_of_ty(r),
                    top_level: false,
                    ty: r,
                });
            }
        }
    }
    result
}

/// Collect every schema-typed [`KclType`] reachable from `ty`'s properties
/// (properties are stored in a map without inherent order, so sort by name
/// for determinism).
fn collect_schema_refs<'a>(ty: &'a KclType, out: &mut Vec<&'a KclType>) {
    let mut keys: Vec<&String> = ty.properties.keys().collect();
    keys.sort();
    for k in keys {
        collect_schema_refs_shallow(&ty.properties[k], out);
    }
}

fn collect_schema_refs_shallow<'a>(ty: &'a KclType, out: &mut Vec<&'a KclType>) {
    match ty.r#type.as_str() {
        "schema" => {
            if !ty.schema_name.is_empty() {
                out.push(ty);
            }
        }
        "list" => {
            if let Some(item) = &ty.item {
                collect_schema_refs_shallow(item, out);
            }
        }
        "dict" => {
            if let Some(item) = &ty.item {
                collect_schema_refs_shallow(item, out);
            }
        }
        "union" => {
            for u in &ty.union_types {
                collect_schema_refs_shallow(u, out);
            }
        }
        _ => {}
    }
}

/// Render the KCL type string of a pb [`KclType`] for documentation output,
/// e.g. `str`, `[int]`, `{str:float}`, `int | str` or the bare schema name.
pub(crate) fn kcl_ty_type_string(ty: &KclType) -> String {
    match ty.r#type.as_str() {
        "list" => format!(
            "[{}]",
            ty.item
                .as_deref()
                .map(kcl_ty_type_string)
                .unwrap_or_default()
        ),
        "dict" => format!(
            "{{{}:{}}}",
            ty.key
                .as_deref()
                .map(kcl_ty_type_string)
                .unwrap_or_default(),
            ty.item
                .as_deref()
                .map(kcl_ty_type_string)
                .unwrap_or_default()
        ),
        "union" => ty
            .union_types
            .iter()
            .map(kcl_ty_type_string)
            .collect::<Vec<_>>()
            .join(" | "),
        "schema" => ty.schema_name.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema_ty(pkg: &str, name: &str) -> KclType {
        KclType {
            r#type: "schema".to_string(),
            schema_name: name.to_string(),
            pkg_path: pkg.to_string(),
            ..Default::default()
        }
    }

    fn mapping(pairs: Vec<(&str, Vec<&str>)>) -> HashMap<String, SchemaTypes> {
        pairs
            .into_iter()
            .map(|(pkg, names)| {
                (
                    pkg.to_string(),
                    SchemaTypes {
                        schema_type: names.into_iter().map(|n| schema_ty(pkg, n)).collect(),
                    },
                )
            })
            .collect()
    }

    #[test]
    fn ordered_schemas_sorts_packages_keeps_list_order() {
        let m = mapping(vec![("bbb", vec!["B1", "B2"]), ("aaa", vec!["A1"])]);
        let ordered = ordered_schemas(&m);
        let got: Vec<(&str, &str)> = ordered
            .iter()
            .map(|s| (s.pkg, s.ty.schema_name.as_str()))
            .collect();
        assert_eq!(got, vec![("aaa", "A1"), ("bbb", "B1"), ("bbb", "B2")]);
    }

    #[test]
    fn require_schemas_rejects_empty_mapping() {
        let m = mapping(vec![]);
        let err = require_schemas(&ordered_schemas(&m)).unwrap_err();
        assert!(
            err.to_string().contains("no schemas found"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn name_resolver_suffixes_cross_package_collisions() {
        let m = mapping(vec![
            ("aaa", vec!["Same", "UniqueA"]),
            ("bbb", vec!["Same"]),
        ]);
        let ordered = ordered_schemas(&m);
        let resolver = SchemaNameResolver::new(&ordered);
        assert_eq!(resolver.name("aaa", "Same"), "Same_aaa");
        assert_eq!(resolver.name("bbb", "Same"), "Same_bbb");
        assert_eq!(resolver.name("aaa", "UniqueA"), "UniqueA");
        // Unknown targets fall back to the bare schema name.
        assert_eq!(resolver.name("ccc", "Else"), "Else");
    }

    #[test]
    fn name_resolver_sanitizes_package_suffixes() {
        let m = mapping(vec![
            ("example.com/pkg.v1", vec!["Same"]),
            ("other", vec!["Same"]),
        ]);
        let ordered = ordered_schemas(&m);
        let resolver = SchemaNameResolver::new(&ordered);
        assert_eq!(
            resolver.name("example.com/pkg.v1", "Same"),
            "Same_example_com_pkg_v1"
        );
        assert_eq!(resolver.name("other", "Same"), "Same_other");
    }

    #[test]
    fn emit_schema_list_appends_referenced_only_closure() {
        // `Derived` references base `External` which is not a top-level
        // schema of the mapping.
        let mut external = schema_ty("ext", "External");
        external.properties.insert(
            "id".to_string(),
            KclType {
                r#type: "int".to_string(),
                ..Default::default()
            },
        );
        let mut derived = schema_ty("main", "Derived");
        derived.base_schema = Some(Box::new(external));
        let m: HashMap<String, SchemaTypes> = [(
            "main".to_string(),
            SchemaTypes {
                schema_type: vec![derived],
            },
        )]
        .into_iter()
        .collect();
        let ordered = ordered_schemas(&m);
        let list = emit_schema_list(&ordered);
        let got: Vec<(&str, bool)> = list
            .iter()
            .map(|s| (s.name.as_str(), s.top_level))
            .collect();
        assert_eq!(got, vec![("Derived", true), ("External", false)]);
    }

    #[test]
    fn kcl_ty_type_string_renders_the_type_matrix() {
        let str_ty = KclType {
            r#type: "str".to_string(),
            ..Default::default()
        };
        assert_eq!(kcl_ty_type_string(&str_ty), "str");
        let list_ty = KclType {
            r#type: "list".to_string(),
            item: Some(Box::new(KclType {
                r#type: "int".to_string(),
                ..Default::default()
            })),
            ..Default::default()
        };
        assert_eq!(kcl_ty_type_string(&list_ty), "[int]");
        let dict_ty = KclType {
            r#type: "dict".to_string(),
            key: Some(Box::new(str_ty.clone())),
            item: Some(Box::new(KclType {
                r#type: "float".to_string(),
                ..Default::default()
            })),
            ..Default::default()
        };
        assert_eq!(kcl_ty_type_string(&dict_ty), "{str:float}");
        let union_ty = KclType {
            r#type: "union".to_string(),
            union_types: vec![
                KclType {
                    r#type: "int".to_string(),
                    ..Default::default()
                },
                str_ty,
            ],
            ..Default::default()
        };
        assert_eq!(kcl_ty_type_string(&union_ty), "int | str");
        assert_eq!(kcl_ty_type_string(&schema_ty("main", "Person")), "Person");
    }
}
