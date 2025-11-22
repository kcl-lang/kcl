// Copyright The KCL Authors. All rights reserved.

use super::context::{DebugModule, LLVMCodeGenContext};
use inkwell::module::Module;

impl<'ctx> LLVMCodeGenContext<'ctx> {
    pub(crate) fn create_debug_module(&self, module: Module<'ctx>) -> DebugModule<'ctx> {
        DebugModule { inner: module }
    }
}
