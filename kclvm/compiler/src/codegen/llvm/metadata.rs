// Copyright The KCL Authors. All rights reserved.

use super::context::{DebugModule, LLVMCodeGenContext};
use crate::codegen::traits::ProgramCodeGen;
use inkwell::module::Module;

impl<'ctx> LLVMCodeGenContext<'ctx> {
    pub(crate) fn create_debug_module(&self, module: Module<'ctx>) -> DebugModule<'ctx> {
        let (dibuilder, compile_unit) = module.create_debug_info_builder(
            true,
            /* language */ inkwell::debug_info::DWARFSourceLanguage::C,
            /* filename */ &self.current_pkgpath(),
            /* directory */ ".",
            /* producer */ "kcl",
            /* is_optimized */ false,
            /* compiler command line flags */ "",
            /* runtime_ver */ 0,
            /* split_name */ "",
            /* kind */ inkwell::debug_info::DWARFEmissionKind::Full,
            /* dwo_id */ 0,
            /* split_debug_inling */ false,
            /* debug_info_for_profiling */ false,
            /* sys_root */ ".",
            "",
        );
        let debug_metadata_version = self.context.i32_type().const_int(3, false);
        module.add_basic_value_flag(
            "Debug Info Version",
            inkwell::module::FlagBehavior::Warning,
            debug_metadata_version,
        );
        DebugModule {
            inner: module,
            dibuilder,
            compile_unit,
        }
    }
}
