//! This package is mainly the implementation of the KCL query tool, mainly including
//! KCL code modification `override` and other implementations. We can call the `override_file`
//! function to modify the file. The main principle is to parse the AST according to the
//! input file name, and according to the ast::OverrideSpec transforms the nodes in the
//! AST, recursively modifying or deleting the values of the nodes in the AST.
pub mod node;
pub mod r#override;
pub mod path;
pub mod query;
pub mod selector;

#[cfg(test)]
mod tests;
mod util;

use anyhow::{Result, anyhow};
use kcl_ast_pretty::print_ast_module;
use kcl_error::diagnostic::Errors;
use kcl_parser::parse_single_file;

use kcl_sema::pre_process::fix_config_expr_nest_attr;
pub use r#override::{apply_override_on_module, apply_overrides};
pub use query::{GetSchemaOption, get_schema_type};

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
/// use kcl_query::override_file;
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
pub fn override_file(
    file: &str,
    specs: &[String],
    import_paths: &[String],
) -> Result<OverrideFileResult> {
    // Parse file to AST module.
    let mut parse_result = match parse_single_file(file, None) {
        Ok(module) => module,
        Err(msg) => return Err(anyhow!("{}", msg)),
    };
    let mut result = false;
    // Override AST module.
    for s in specs {
        if apply_override_on_module(&mut parse_result.module, s, import_paths)? {
            result = true;
        }
    }

    // Transform config expr to simplify the config path query and override.
    fix_config_expr_nest_attr(&mut parse_result.module);
    // Print AST module.
    if result {
        let code_str = print_ast_module(&parse_result.module);
        std::fs::write(file, code_str)?
    }
    Ok(OverrideFileResult {
        result,
        parse_errors: parse_result.errors,
    })
}

pub struct OverrideFileResult {
    pub result: bool,
    pub parse_errors: Errors,
}
