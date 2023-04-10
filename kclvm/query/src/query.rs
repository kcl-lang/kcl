use std::{cell::RefCell, rc::Rc, sync::Arc};

use anyhow::Result;
use indexmap::IndexMap;
use kclvm_parser::{load_program, LoadProgramOptions, ParseSession};
use kclvm_sema::{
    resolver::{resolve_program, scope::Scope},
    ty::SchemaType,
};

/// Get schema type kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GetSchemaOption {
    /// Get schema instances.
    Instances,
    /// Get schema definitions.
    Definitions,
    /// Get schema instances and definitions
    All,
}

impl Default for GetSchemaOption {
    fn default() -> Self {
        GetSchemaOption::All
    }
}

/// Get schema types from a kcl file or code.
///
/// # Parameters
/// file: [&str]. The kcl filename.
///
/// code: [Option<&str>]. The kcl code string
///
/// schema_name: [Option<&str>]. The schema name, when the schema name is empty, all schemas are returned.
///
/// # Examples
///
/// ```
/// use kclvm_query::query::{get_schema_type, GetSchemaOption};
///
/// let file = "schema.k";
/// let code = r#"
/// import units
///
/// schema Person:
///     name: str
///     age: int
///     size?: units.NumberMultiplier = 1Mi
///
/// person = Person {
///     name = "Alice"
///     age = 18
/// }
/// "#;
/// // Get all schema
/// let types = get_schema_type(file, Some(code), None, GetSchemaOption::All).unwrap();
/// assert_eq!(types.len(), 2);
/// assert_eq!(types[0].name, "Person");
/// assert_eq!(types[1].name, "Person");
/// assert_eq!(types["Person"].name, "Person");
/// assert_eq!(types["person"].name, "Person");
///
/// let types = get_schema_type(file, Some(code), None, GetSchemaOption::Instances).unwrap();
/// assert_eq!(types.len(), 1);
/// assert_eq!(types[0].name, "Person");
/// assert_eq!(types["person"].name, "Person");
///
/// let types = get_schema_type(file, Some(code), None, GetSchemaOption::Definitions).unwrap();
/// assert_eq!(types.len(), 1);
/// assert_eq!(types[0].name, "Person");
/// assert_eq!(types["Person"].name, "Person");
/// assert_eq!(types["Person"].attrs["name"].ty.ty_str(), "str");
/// assert_eq!(types["Person"].attrs["age"].ty.ty_str(), "int");
/// assert_eq!(types["Person"].attrs["size"].ty.ty_str(), "number_multiplier");
/// ```
pub fn get_schema_type(
    file: &str,
    code: Option<&str>,
    schema_name: Option<&str>,
    opt: GetSchemaOption,
) -> Result<IndexMap<String, SchemaType>> {
    let mut result = IndexMap::new();
    let scope = resolve_file(file, code)?;
    for (name, o) in &scope.borrow().elems {
        if o.borrow().ty.is_schema() {
            let schema_ty = o.borrow().ty.into_schema_type();
            if opt == GetSchemaOption::All
                || (opt == GetSchemaOption::Definitions && !schema_ty.is_instance)
                || (opt == GetSchemaOption::Instances && schema_ty.is_instance)
            {
                // Schema name filter
                match schema_name {
                    Some(schema_name) => {
                        if schema_name == name {
                            result.insert(name.to_string(), schema_ty);
                        }
                    }
                    None => {
                        result.insert(name.to_string(), schema_ty);
                    }
                }
            }
        }
    }
    Ok(result)
}

fn resolve_file(file: &str, code: Option<&str>) -> Result<Rc<RefCell<Scope>>> {
    let sess = Arc::new(ParseSession::default());
    let mut program = match load_program(
        sess,
        &[file],
        code.map(|c| LoadProgramOptions {
            k_code_list: vec![c.to_string()],
            ..Default::default()
        }),
    ) {
        Ok(p) => p,
        Err(err) => {
            return Err(anyhow::anyhow!("{err}"));
        }
    };
    let scope = resolve_program(&mut program);
    match scope.main_scope() {
        Some(scope) => Ok(scope.clone()),
        None => Err(anyhow::anyhow!("main scope is not found")),
    }
}
