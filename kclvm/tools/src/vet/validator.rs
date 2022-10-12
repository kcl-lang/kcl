//! KCL-Vet can use KCL to validate the content of json or yaml files.
//!
//! The entry point of KCL-Vet is method `validate`, for more information, see doc above method `validate`.
//!
//! The main principle consists of three parts:
//!
//! - Validation rules for validating file contents are defined in KCL statment.
//! - Convert the json or yaml file to be verified into a KCL assign expression.
//! - Combine KCL statment and KCL expression into a KCL program,
//!   and the KCL program is checked by the KCLVM compiler.
//!
//! For example.
//!
//! 1. If the json file to be verified is as follows:
//! (kclvm/tools/src/vet/test_datas/validate_cases/test.json)
//!
//! ```ignore
//! {
//!     "name": "Alice",
//!     "age": 18,
//!     "message": "This is Alice"
//! }
//! ```
//!
//! 2. You can define KCL like below and define validation rules in check block.
//! (kclvm/tools/src/vet/test_datas/validate_cases/test.k)
//!
//! ```ignore
//! schema User:
//!     name: str
//!     age: int
//!     message?: str
//!
//!     check:
//!         name == "Alice"
//!         age > 10
//! ```
//!
//! 3. The json file mentioned in 1 will generate the following kcl expression:
//!
//! ```ignore
//! value = User {
//!     name: "Alice",
//!     age: 18,
//!     message: "This is Alice"
//! }
//! ```
//!
//! 4. Finally, a KCL program like the following will be handed over to KCLVM to compile and check for problems.
//!
//! ```ignore
//! value = User {
//!     name: "Alice",
//!     age: 18,
//!     message: "This is Alice"
//! }
//!
//! schema User:
//!     name: str
//!     age: int
//!     message?: str
//!
//!     check:
//!         name == "Alice"
//!         age > 10
//! ```
use std::collections::HashMap;

use crate::util::loader::LoaderKind;

use super::expr_builder::ExprBuilder;
use kclvm_ast::{
    ast::{
        AssignStmt, Expr, ExprContext, Identifier, Module, Node, NodeRef, Program, SchemaStmt, Stmt,
    },
    node_ref,
};
use kclvm_runner::{execute, ExecProgramArgs};

const TMP_FILE: &str = "validationTempKCLCode.k";

/// Validate the data string using the schema code string, when the parameter
/// `schema` is omitted, use the first schema appeared in the kcl code, when the schema
/// not found, raise an schema not found error.
///
/// Returns a string result denoting whether validating success, raise an error
/// when validating failed because of the file not found error, schema not found
/// error, syntax error, check error, etc.
///
/// When the content of the json file conforms to the rules, a normal kcl expression will be returned.
///
/// # Examples
///
/// 1. If you want to verify the following json file.
/// (kclvm/tools/src/vet/test_datas/validate_cases/test.json)
/// ```ignore
/// {
///     "name": "Alice",
///     "age": 18,
///     "message": "This is Alice"
/// }
/// ```
///
/// 2. First, you can create a KCL schema and write validation rules.
/// (kclvm/tools/src/vet/test_datas/validate_cases/test.k)
/// ```ignore
/// schema User:
///     name: str
///     age: int
///     message?: str
///
///     check:
///         name == "Alice"
///         age > 10
/// ```
///
/// 3. Second, you can call this method as follows to validate the content of the json file with the kcl file.
/// ```rust
/// # use kclvm_tools::vet::validator::validate;
/// # use std::path::PathBuf;
/// # use kclvm_tools::util::loader::LoaderKind;
///
/// // First get the file path of the file to be verified.
/// let mut validated_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
/// validated_file_path.push("src/vet/test_datas/validate_cases/test.json");
/// let validated_file_path = validated_file_path.to_str().unwrap();
///
/// // Then get the path to the KCL file.
/// let mut kcl_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
/// kcl_file_path.push("src/vet/test_datas/validate_cases/test.k");
/// let kcl_file_path = Some(kcl_file_path.to_str().unwrap());
///
/// // Get the name of the schema defined in the kcl file
/// let schema_name = Some("User".to_string());
///
/// // Define the name of an attribute.
/// // The name of this property is related to the rules in the KCL file.
/// let attr_name = "value";
///
/// // Define the kind of file you want to validate.
/// let kind = LoaderKind::JSON;
///
/// // One of the KCL file path or the content of the KCL file is enough.
/// let result = validate(schema_name, attr_name, validated_file_path.to_string(), kind, kcl_file_path, None);
/// ```
///
/// The json file used above conforms to the schema rules, so the content of `result` you get is :
/// ```ignore
/// {
/// "value": {
///     "name": "Alice",
///     "age": 18,
///     "message": "This is Alice",
///         "__settings__": {
///             "output_type": "INLINE",
///             "__schema_type__": "__main__.User"
///         }
///     }
/// }
/// ```
///
/// If you change the content of the above json file to :
/// ```ignore
/// {
///     "name": "Tom",
///     "age": 18,
///     "message": "This is Alice"
/// }
/// ```
///
/// You will get an error message like this:
/// ```ignore
/// {
///     "__kcl_PanicInfo__": true,
///     "rust_file": "runtime/src/value/api.rs",
///     "rust_line": 2203,
///     "rust_col": 9,
///     "kcl_pkgpath": "__main__",
///     "kcl_file": "kclvm/tools/src/vet/test_datas/invalid_validate_cases/test.json",
///     "kcl_line": 7,
///     "kcl_col": 0,
///     "kcl_arg_msg": "Check failed on the condition",
///     "kcl_config_meta_file": "",
///     "kcl_config_meta_line": 1,
///     "kcl_config_meta_col": 1,
///     "kcl_config_meta_arg_msg": "Instance check failed",
///     "message": "",
///     "err_type_code": 17,
///     "is_warning": false
/// }
/// ```
pub fn validate(
    schema_name: Option<String>,
    attribute_name: &str,
    validated_file_path: String,
    validated_file_kind: LoaderKind,
    kcl_path: Option<&str>,
    kcl_code: Option<String>,
) -> String {
    let k_path = match kcl_path {
        Some(path) => path,
        None => TMP_FILE,
    };

    let mut module: Module = match kclvm_parser::parse_file(&k_path, kcl_code) {
        Ok(ast_m) => ast_m,
        Err(err_msg) => return err_msg,
    };

    let schemas = filter_schema_stmt(&module);
    let schema_name = match schema_name {
        Some(name) => Some(name),
        None => match schemas.get(0) {
            Some(schema) => Some(schema.name.node.clone()),
            None => None,
        },
    };

    let expr_builder =
        match ExprBuilder::new_with_file_path(validated_file_kind, validated_file_path) {
            Ok(builder) => builder,
            Err(_) => return "Failed to load validated file.".to_string(),
        };

    let validated_expr = match expr_builder.build(schema_name) {
        Ok(expr) => expr,
        Err(_) => return "Failed to load validated file.".to_string(),
    };

    let assign_stmt = build_assign(attribute_name, validated_expr);

    module.body.insert(0, assign_stmt);

    match eval_ast(module) {
        Ok(res) => res,
        Err(err) => err,
    }
}

fn build_assign(attr_name: &str, node: NodeRef<Expr>) -> NodeRef<Stmt> {
    node_ref!(Stmt::Assign(AssignStmt {
        targets: vec![node_ref!(Identifier {
            names: vec![attr_name.to_string()],
            pkgpath: String::new(),
            ctx: ExprContext::Store,
        })],
        value: node,
        type_annotation: None,
        ty: None,
    }))
}

const MAIN_PKG_NAME: &str = "__main__";

fn eval_ast(mut m: Module) -> Result<String, String> {
    m.pkg = MAIN_PKG_NAME.to_string();

    let mut pkgs = HashMap::new();
    pkgs.insert(MAIN_PKG_NAME.to_string(), vec![m]);

    let prog = Program {
        root: MAIN_PKG_NAME.to_string(),
        main: MAIN_PKG_NAME.to_string(),
        pkgs,
        cmd_args: vec![],
        cmd_overrides: vec![],
    };

    execute(prog, 0, &ExecProgramArgs::default())
}

fn filter_schema_stmt(module: &Module) -> Vec<&SchemaStmt> {
    let mut result = vec![];
    for stmt in &module.body {
        if let Stmt::Schema(s) = &stmt.node {
            result.push(s);
        }
    }
    result
}
