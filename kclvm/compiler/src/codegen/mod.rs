//! Copyright 2021 The KCL Authors. All rights reserved.

use indexmap::IndexMap;
use kclvm_ast::ast;

mod abi;
pub mod error;
#[cfg(feature = "llvm")]
pub mod llvm;
mod traits;

/// The kclvm runner main function name.
pub const MODULE_NAME: &str = "kclvm_main";
/// The kclvm runner main function entry block name.
pub const ENTRY_NAME: &str = "entry";
/// The kclvm runtime value type name.
pub const VALUE_TYPE_NAME: &str = "kclvm_value_ref_t";
/// The kclvm runtime context type name.
pub const CONTEXT_TYPE_NAME: &str = "kclvm_context_t";
/// Package init function name suffix
pub const PKG_INIT_FUNCTION_SUFFIX: &str = "init";
/// Global level
pub const GLOBAL_LEVEL: usize = 1;
/// Inner level
pub const INNER_LEVEL: usize = 2;
/// Global variable alignment
pub const GLOBAL_VAL_ALIGNMENT: u32 = 8;
/// Object file type format suffix.
#[cfg(target_os = "windows")]
pub const OBJECT_FILE_SUFFIX: &str = ".obj";
#[cfg(not(target_os = "windows"))]
pub const OBJECT_FILE_SUFFIX: &str = ".o";
/// LLVM IR text format suffix .ll
pub const LL_FILE_SUFFIX: &str = ".ll";

/// CodeGenContext is a trait used by the compiler to emit code to different targets.
pub trait CodeGenContext: traits::ProgramCodeGen {
    fn emit(&self, opt: &EmitOptions) -> Result<(), Box<dyn std::error::Error>>;
}

/// EmitOptions represents the general emit options
#[derive(Debug, Default)]
pub struct EmitOptions<'a> {
    /// Path to load exist module, if not set, create an empty module.
    pub from_path: Option<&'a str>,
    /// Path to emit module.
    pub emit_path: Option<&'a str>,
    /// no_link indicates whether to link the generated code of different KCL packages to the same module.
    pub no_link: bool,
}

/// Emit code with the options using CodeGenContext.
pub fn emit_code_with(
    ctx: impl CodeGenContext,
    opt: &EmitOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    ctx.emit(opt)
}

/// Generate LLVM IR of KCL ast module.
#[inline]
pub fn emit_code(
    program: &ast::Program,
    workdir: String,
    import_names: IndexMap<String, IndexMap<String, String>>,
    opts: &EmitOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "llvm")]
    {
        llvm::emit_code(program, workdir, import_names, opts)
    }
    #[cfg(not(feature = "llvm"))]
    {
        let _ = program;
        let _ = workdir;
        let _ = import_names;
        let _ = opts;
        Err("error: llvm feature is not enabled. Note: Set KCL_FAST_EVAL=1 or rebuild the crate with the llvm feature.".to_string().into())
    }
}
