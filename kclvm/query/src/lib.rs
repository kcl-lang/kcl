//! This package is mainly the implementation of the KCL query tool, mainly including
//! KCL code modification `override` and other implementations. We can call the `override_file`
//! function to modify the file. The main principle is to parse the AST according to the
//! input file name, and according to the ast::OverrideSpec transforms the nodes in the
//! AST, recursively modifying or deleting the values of the nodes in the AST.
pub mod r#override;
pub mod path;
pub mod query;
pub mod selector;

#[cfg(test)]
mod tests;
mod util;

use anyhow::{anyhow, Result};
use kclvm_ast::ast;
use kclvm_ast_pretty::print_ast_module;
use kclvm_parser::parse_file;

pub use query::{get_schema_type, GetSchemaOption};
pub use r#override::{apply_override_on_module, apply_overrides};

use self::r#override::parse_override_spec;

/// Override and rewrite a file with override specifications. Please note that this is an external user API,
/// and it can directly modify the KCL file in place.
///
/// # Parameters
///
/// `file`: [&str]
///     The File that need to be overridden
///
/// `specs`: &\[[String]\]
///     List of specs that need to be overridden.
///     Each spec string satisfies the form: <pkgpath>:<field_path>=<filed_value> or <pkgpath>:<field_path>-
///     When the pkgpath is '__main__', `<pkgpath>:` can be omitted.
///
/// `import_paths`: &\[[String]\]
///     List of import paths that are need to be added.
///
/// # Returns
///
/// result: [Result<bool>]
///     Whether the file has been modified.
///
/// # Examples
///
/// ```no_run
/// use kclvm_query::override_file;
///
/// let result = override_file(
///     "test.k",
///     &["alice.age=18".to_string()],
///     &[]
/// ).unwrap();
/// ```
///
/// - test.k (before override)
///
/// ```kcl
/// schema Person:
///     age: int
///
/// alice = Person {
///     age = 10
/// }
/// ```
///
/// - test.k (after override)
///
/// ```kcl
/// schema Person:
///     age: int
///
/// alice = Person {
///     age = 18
/// }
/// ```
pub fn override_file(file: &str, specs: &[String], import_paths: &[String]) -> Result<bool> {
    // Parse override spec strings.
    let overrides = specs
        .iter()
        .map(|s| parse_override_spec(s))
        .filter_map(Result::ok)
        .collect::<Vec<ast::OverrideSpec>>();
    // Parse file to AST module.
    let mut module = match parse_file(file, None) {
        Ok(module) => module.module,
        Err(msg) => return Err(anyhow!("{}", msg)),
    };
    let mut result = false;
    // Override AST module.
    for o in &overrides {
        if apply_override_on_module(&mut module, o, import_paths)? {
            result = true;
        }
    }
    // Print AST module.
    if result {
        let code_str = print_ast_module(&module);
        std::fs::write(file, code_str)?
    }
    Ok(result)
}
