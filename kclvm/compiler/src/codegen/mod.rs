//! Copyright 2021 The KCL Authors. All rights reserved.

mod abi;
pub mod error;
pub mod llvm;
mod traits;

/// The kclvm runner main function name.
pub(crate) const MODULE_NAME: &str = "kclvm_main";
/// The kclvm runner main function entry block name.
pub(crate) const ENTRY_NAME: &str = "entry";
/// The kclvm runtime value type name.
pub(crate) const VALUE_TYPE_NAME: &str = "kclvm_value_ref_t";
/// The kclvm runtime context type name.
pub(crate) const CONTEXT_TYPE_NAME: &str = "kclvm_context_t";
/// Package init function name suffix
pub(crate) const PKG_INIT_FUNCTION_SUFFIX: &str = "init";
/// Global level
pub(crate) const GLOBAL_LEVEL: usize = 1;
/// Inner level
pub(crate) const INNER_LEVEL: usize = 2;
/// Global variable alignment
pub(crate) const GLOBAL_VAL_ALIGNMENT: u32 = 8;

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
pub fn emit_code(
    ctx: impl CodeGenContext,
    opt: &EmitOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    ctx.emit(opt)
}
