// Copyright The KCL Authors. All rights reserved.

use kclvm_ast::ast;
use kclvm_runtime::ApiFunc;
use std::str;

use crate::codegen::traits::ValueMethods;

use super::context::LLVMCodeGenContext;

/*
 * Temporal functions
 */

/// Update runtime context pkgpath
pub(crate) fn update_ctx_pkgpath(gen: &LLVMCodeGenContext, pkgpath: &str) {
    gen.build_void_call(
        &ApiFunc::kclvm_context_set_kcl_pkgpath.name(),
        &[
            gen.current_runtime_ctx_ptr(),
            gen.native_global_string_value(pkgpath),
        ],
    );
}

/// Update runtime context filename
pub(crate) fn update_ctx_filename<'ctx, T>(
    gen: &'ctx LLVMCodeGenContext,
    node: &'ctx ast::Node<T>,
) {
    if !node.filename.is_empty() {
        gen.build_void_call(
            &ApiFunc::kclvm_context_set_kcl_filename.name(),
            &[
                gen.current_runtime_ctx_ptr(),
                gen.native_global_string_value(&node.filename),
            ],
        );
    }
}

/// Update runtime context line and column
pub(crate) fn update_ctx_line_col<'ctx, T>(
    gen: &'ctx LLVMCodeGenContext,
    node: &'ctx ast::Node<T>,
) {
    let mut current_line = gen.current_line.borrow_mut();
    if node.line != *current_line {
        *current_line = node.line;
        gen.build_void_call(
            &ApiFunc::kclvm_context_set_kcl_line_col.name(),
            &[
                gen.current_runtime_ctx_ptr(),
                gen.native_int_value(node.line as i32),
                gen.native_int_value(0),
            ],
        );
    }
}

/// Update runtime context line and column
pub(crate) fn update_ctx_current_line(gen: &LLVMCodeGenContext) {
    let current_line = gen.current_line.borrow_mut();
    gen.build_void_call(
        &ApiFunc::kclvm_context_set_kcl_line_col.name(),
        &[
            gen.current_runtime_ctx_ptr(),
            gen.native_int_value(*current_line as i32),
            gen.native_int_value(0),
        ],
    );
}

/// Reset target vars
pub(crate) fn reset_target_vars(gen: &LLVMCodeGenContext) {
    gen.target_vars.borrow_mut().clear();
    gen.target_vars.borrow_mut().push("".to_string());
}
