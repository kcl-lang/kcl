// Copyright 2021 The KCL Authors. All rights reserved.

use indexmap::{IndexMap, IndexSet};
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::debug_info::{DICompileUnit, DebugInfoBuilder};
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::module::{Linkage, Module};
use inkwell::support::LLVMString;
use inkwell::targets::{CodeModel, FileType, RelocMode};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue, PointerValue,
};
use inkwell::{AddressSpace, IntPredicate};
use phf::{phf_map, Map};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::Path;
use std::rc::Rc;
use std::str;

use kclvm_ast::ast;
use kclvm_error::*;
use kclvm_runtime::{ApiFunc, MAIN_PKG_PATH, PKG_PATH_PREFIX};
use kclvm_sema::builtin;
use kclvm_sema::pkgpath_without_prefix;
use kclvm_sema::plugin;

use crate::codegen::abi::Align;
use crate::codegen::{error as kcl_error, EmitOptions};
use crate::codegen::{
    traits::*, ENTRY_NAME, GLOBAL_VAL_ALIGNMENT, MODULE_NAME, PKG_INIT_FUNCTION_SUFFIX,
};
use crate::codegen::{CodeGenContext, GLOBAL_LEVEL};
use crate::value;

use super::OBJECT_FILE_SUFFIX;

/// SCALAR_KEY denotes the temp scalar key for the global variable json plan process.
const SCALAR_KEY: &str = "";
/// Float type string width mapping
pub const FLOAT_TYPE_WIDTH_MAPPING: Map<&str, usize> = phf_map! {
    "half" => 16,
    "float" => 32,
    "double" => 64,
    "x86_fp80" => 80,
    "ppc_fp128" => 128,
    "fp128" => 128,
};

/// The compiler function result
pub type CompileResult<'a> = Result<BasicValueEnum<'a>, kcl_error::KCLError>;

/// The compiler scope.
#[derive(Debug, Default)]
pub struct Scope<'ctx> {
    /// Scalars denotes the expression statement values without attribute.
    pub scalars: RefCell<Vec<BasicValueEnum<'ctx>>>,
    /// schema_scalar_idx denotes whether a schema exists in the scalar list.
    pub schema_scalar_idx: RefCell<usize>,
    /// Scope normal variables
    pub variables: RefCell<IndexMap<String, PointerValue<'ctx>>>,
    /// Scope closures referenced by internal scope.
    pub closures: RefCell<IndexMap<String, PointerValue<'ctx>>>,
    /// Potential arguments in the current scope, such as schema/lambda arguments.
    pub arguments: RefCell<IndexSet<String>>,
}

/// Schema or Global internal order independent computation backtracking meta information.
pub struct BacktrackMeta {
    pub target: String,
    pub level: usize,
    pub count: usize,
    pub stop: bool,
}

/// The LLVM code generator
pub struct LLVMCodeGenContext<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub program: &'ctx ast::Program,
    pub functions: RefCell<Vec<Rc<FunctionValue<'ctx>>>>,
    pub imported: RefCell<HashSet<String>>,
    pub schema_stack: RefCell<Vec<value::SchemaType>>,
    pub lambda_stack: RefCell<Vec<usize>>,
    pub schema_expr_stack: RefCell<Vec<()>>,
    pub pkgpath_stack: RefCell<Vec<String>>,
    pub filename_stack: RefCell<Vec<String>>,
    /// Package scope to store variable pointers.
    pub pkg_scopes: RefCell<HashMap<String, Vec<Rc<Scope<'ctx>>>>>,
    /// Local variables in the loop.
    pub local_vars: RefCell<HashSet<String>>,
    /// The names of possible assignment objects for the current instruction.
    pub target_vars: RefCell<Vec<String>>,
    /// Global string caches
    pub global_strings: RefCell<IndexMap<String, IndexMap<String, PointerValue<'ctx>>>>,
    /// Global variable pointers cross different packages.
    pub global_vars: RefCell<IndexMap<String, IndexMap<String, PointerValue<'ctx>>>>,
    /// The filename of the source file corresponding to the current instruction
    pub current_filename: RefCell<String>,
    /// The line number of the source file corresponding to the current instruction
    pub current_line: RefCell<u64>,
    /// Error handler to store compile errors.
    pub handler: RefCell<Handler>,
    /// Schema attr backtrack meta
    pub backtrack_meta: RefCell<Option<BacktrackMeta>>,
    /// Import names mapping
    pub import_names: IndexMap<String, IndexMap<String, String>>,
    /// No link mode
    pub no_link: bool,
    /// Debug mode
    pub debug: bool,
    /// Program modules according to AST modules
    pub modules: RefCell<HashMap<String, RefCell<DebugModule<'ctx>>>>,
    /// Program workdir
    pub workdir: String,
}

/// LLVM module with debug info builder and compile unit.
pub struct DebugModule<'ctx> {
    pub inner: Module<'ctx>,
    pub dibuilder: DebugInfoBuilder<'ctx>,
    pub compile_unit: DICompileUnit<'ctx>,
}

impl<'ctx> CodeGenObject for BasicValueEnum<'ctx> {}

impl<'ctx> CodeGenObject for BasicTypeEnum<'ctx> {}

impl<'ctx> BackendTypes for LLVMCodeGenContext<'ctx> {
    type Value = BasicValueEnum<'ctx>;
    type Type = BasicTypeEnum<'ctx>;
    type BasicBlock = BasicBlock<'ctx>;
    type Function = FunctionValue<'ctx>;
    type FunctionLet = FunctionType<'ctx>;
}

impl<'ctx> BuilderMethods for LLVMCodeGenContext<'ctx> {
    /// SSA append a basic block named `name`.
    #[inline]
    fn append_block(&self, name: &str) -> Self::BasicBlock {
        let cur_func = self.current_function();
        self.context.append_basic_block(cur_func, name)
    }
    /// SSA switch to the block.
    #[inline]
    fn switch_to_block(&self, block: Self::BasicBlock) {
        self.builder.position_at_end(block);
    }
    /// SSA alloca instruction.
    #[inline]
    fn alloca(&self, ty: Self::Type, name: &str, _align: Option<Align>) -> Self::Value {
        self.builder.build_alloca(ty, name).into()
    }
    /// SSA array alloca instruction.
    #[inline]
    fn array_alloca(
        &self,
        ty: Self::Type,
        len: Self::Value,
        name: &str,
        _align: Align,
    ) -> Self::Value {
        self.builder
            .build_array_alloca(ty, len.into_int_value(), name)
            .into()
    }
    /// SSA ret instruction.
    #[inline]
    fn ret_void(&self) {
        self.builder.build_return(None);
    }
    /// SSA ret instruction with returned value.
    #[inline]
    fn ret(&self, v: Self::Value) {
        self.builder.build_return(Some(&v));
    }
    /// SSA br instruction.
    #[inline]
    fn br(&self, dest: Self::BasicBlock) {
        self.builder.build_unconditional_branch(dest);
    }
    /// SSA cond br instruction.
    #[inline]
    fn cond_br(&self, cond: Self::Value, then_bb: Self::BasicBlock, else_bb: Self::BasicBlock) {
        self.builder
            .build_conditional_branch(cond.into_int_value(), then_bb, else_bb);
    }
    /// SSA select instruction.
    #[inline]
    fn select(
        &self,
        cond: Self::Value,
        then_val: Self::Value,
        else_val: Self::Value,
    ) -> Self::Value {
        self.builder
            .build_select(cond.into_int_value(), then_val, else_val, "")
    }
    /// SSA va arg instruction.
    #[inline]
    fn va_arg(&self, list: Self::Value, ty: Self::Type) -> Self::Value {
        self.builder.build_va_arg(list.into_pointer_value(), ty, "")
    }
    /// SSA extract element instruction.
    #[inline]
    fn extract_element(&self, vec: Self::Value, idx: Self::Value) -> Self::Value {
        self.builder
            .build_extract_element(vec.into_vector_value(), idx.into_int_value(), "")
    }
    /// SSA extract value instruction.
    #[inline]
    fn extract_value(&self, agg_val: Self::Value, idx: u32) -> Self::Value {
        self.builder
            .build_extract_value(agg_val.into_array_value(), idx, "")
            .expect(kcl_error::INTERNAL_ERROR_MSG)
    }
    /// SSA insert value instruction.
    #[inline]
    fn insert_value(&self, agg_val: Self::Value, elt: Self::Value, idx: u32) -> Self::Value {
        self.builder
            .build_insert_value(agg_val.into_array_value(), elt, idx, "")
            .expect(kcl_error::INTERNAL_ERROR_MSG)
            .into_array_value()
            .into()
    }
    /// SSA function invoke instruction.
    #[inline]
    fn invoke(
        &self,
        _ty: Self::Type,
        fn_value: Self::Function,
        args: &[Self::Value],
        then: Self::BasicBlock,
        catch: Self::BasicBlock,
    ) -> Self::Value {
        self.builder
            .build_invoke(fn_value, args, then, catch, "")
            .try_as_basic_value()
            .expect_left(kcl_error::INTERNAL_ERROR_MSG)
    }
    /// SSA function call instruction.
    #[inline]
    fn call(&self, _ty: Self::Type, fn_value: Self::Function, args: &[Self::Value]) -> Self::Value {
        let args: Vec<BasicMetadataValueEnum> = args.iter().map(|arg| (*arg).into()).collect();
        self.builder
            .build_call(fn_value, &args, "")
            .try_as_basic_value()
            .left()
            .expect(kcl_error::FUNCTION_RETURN_VALUE_NOT_FOUND_MSG)
    }
    /// SSA load instruction.
    #[inline]
    fn load(&self, ptr: Self::Value, name: &str) -> Self::Value {
        self.builder.build_load(ptr.into_pointer_value(), name)
    }
    /// SSA store instruction.
    #[inline]
    fn store(&self, ptr: Self::Value, val: Self::Value) {
        self.builder.build_store(ptr.into_pointer_value(), val);
    }
    /// SSA gep instruction.
    #[inline]
    fn gep(&self, _ty: Self::Type, ptr: Self::Value, indices: &[Self::Value]) -> Self::Value {
        let ordered_indexes: Vec<IntValue> = indices.iter().map(|v| v.into_int_value()).collect();
        unsafe {
            self.builder
                .build_gep(ptr.into_pointer_value(), &ordered_indexes, "")
                .into()
        }
    }
    /// SSA inbounds gep instruction.
    #[inline]
    fn inbounds_gep(
        &self,
        _ty: Self::Type,
        ptr: Self::Value,
        indices: &[Self::Value],
    ) -> Self::Value {
        let ordered_indexes: Vec<IntValue> = indices.iter().map(|v| v.into_int_value()).collect();
        unsafe {
            self.builder
                .build_in_bounds_gep(ptr.into_pointer_value(), &ordered_indexes, "")
                .into()
        }
    }
    /// SSA struct gep instruction.
    #[inline]
    fn struct_gep(&self, _ty: Self::Type, ptr: Self::Value, idx: u32) -> Self::Value {
        self.builder
            .build_struct_gep(ptr.into_pointer_value(), idx, "")
            .expect(kcl_error::INTERNAL_ERROR_MSG)
            .into()
    }
    /// SSA cast pointer to int.
    #[inline]
    fn ptr_to_int(&self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.builder
            .build_ptr_to_int(val.into_pointer_value(), dest_ty.into_int_type(), "")
            .into()
    }
    /// SSA cast int to pointer.
    #[inline]
    fn int_to_ptr(&self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.builder
            .build_int_to_ptr(val.into_int_value(), dest_ty.into_pointer_type(), "")
            .into()
    }
    /// SSA bit cast.
    #[inline]
    fn bit_cast(&self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.builder.build_bitcast(val, dest_ty, "")
    }
    /// SSA int cast.
    #[inline]
    fn int_cast(&self, val: Self::Value, dest_ty: Self::Type, _is_signed: bool) -> Self::Value {
        self.builder
            .build_int_cast(val.into_int_value(), dest_ty.into_int_type(), "")
            .into()
    }
    /// SSA pointer cast.
    #[inline]
    fn ptr_cast(&self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.builder
            .build_pointer_cast(val.into_pointer_value(), dest_ty.into_pointer_type(), "")
            .into()
    }
    /// Lookup a known function named `name`.
    fn lookup_function(&self, name: &str) -> Self::Function {
        if self.no_link {
            let pkgpath = self.current_pkgpath();
            let modules = self.modules.borrow();
            let msg = format!("pkgpath {} is not found", pkgpath);
            let module = &modules.get(&pkgpath).expect(&msg).borrow().inner;
            if let Some(function) = module.get_function(name) {
                function
            } else {
                let function = self
                    .module
                    .get_function(name)
                    .unwrap_or_else(|| panic!("known function {} not found", name));
                let fn_type = function.get_type();
                module.add_function(name, fn_type, Some(Linkage::External))
            }
        } else {
            self.module
                .get_function(name)
                .unwrap_or_else(|| panic!("known function {} not found", name))
        }
    }
    /// Add a function named `name`.
    fn add_function(&self, name: &str) -> Self::Function {
        let fn_ty = self.function_type();
        if self.no_link {
            let pkgpath = self.current_pkgpath();
            let msg = format!("pkgpath {} is not found", pkgpath);
            let modules = self.modules.borrow_mut();
            let module = &modules.get(&pkgpath).expect(&msg).borrow_mut().inner;
            module.add_function(name, fn_ty, None)
        } else {
            self.module.add_function(name, fn_ty, None)
        }
    }
}

/* Value methods */

impl<'ctx> ValueMethods for LLVMCodeGenContext<'ctx> {
    /// Construct a 64-bit int value using i64
    fn int_value(&self, v: i64) -> Self::Value {
        let i64_type = self.context.i64_type();
        self.build_call(
            &ApiFunc::kclvm_value_Int.name(),
            &[
                self.current_runtime_ctx_ptr(),
                i64_type.const_int(v as u64, false).into(),
            ],
        )
    }

    /// Construct a 64-bit float value using f64
    fn float_value(&self, v: f64) -> Self::Value {
        let f64_type = self.context.f64_type();
        self.build_call(
            &ApiFunc::kclvm_value_Float.name(),
            &[
                self.current_runtime_ctx_ptr(),
                f64_type.const_float(v).into(),
            ],
        )
    }

    /// Construct a string value using &str
    fn string_value(&self, v: &str) -> Self::Value {
        let string_ptr_value = self.native_global_string(v, "");
        self.build_call(
            &ApiFunc::kclvm_value_Str.name(),
            &[self.current_runtime_ctx_ptr(), string_ptr_value.into()],
        )
    }

    /// Construct a bool value
    fn bool_value(&self, v: bool) -> Self::Value {
        let i8_type = self.context.i8_type();
        self.build_call(
            &ApiFunc::kclvm_value_Bool.name(),
            &[
                self.current_runtime_ctx_ptr(),
                i8_type.const_int(v as u64, false).into(),
            ],
        )
    }

    /// Construct a None value
    fn none_value(&self) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_None.name(),
            &[self.current_runtime_ctx_ptr()],
        )
    }

    /// Construct a Undefined value
    fn undefined_value(&self) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_Undefined.name(),
            &[self.current_runtime_ctx_ptr()],
        )
    }

    /// Construct a empty kcl list value
    fn list_value(&self) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_List.name(),
            &[self.current_runtime_ctx_ptr()],
        )
    }

    /// Construct a list value with `n` elements
    fn list_values(&self, values: &[Self::Value]) -> Self::Value {
        let mut args = vec![self.current_runtime_ctx_ptr()];
        for value in values {
            args.push(*value);
        }
        self.build_call(
            &format!("{}{}", ApiFunc::kclvm_value_List.name(), values.len()),
            args.as_slice(),
        )
    }

    /// Construct a empty kcl dict value.
    fn dict_value(&self) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_Dict.name(),
            &[self.current_runtime_ctx_ptr()],
        )
    }

    /// Construct a unit value
    fn unit_value(&self, v: f64, raw: i64, unit: &str) -> Self::Value {
        let f64_type = self.context.f64_type();
        let i64_type = self.context.i64_type();
        let unit_native_str = self.native_global_string(unit, "");
        self.build_call(
            &ApiFunc::kclvm_value_Unit.name(),
            &[
                self.current_runtime_ctx_ptr(),
                f64_type.const_float(v).into(),
                i64_type.const_int(raw as u64, false).into(),
                unit_native_str.into(),
            ],
        )
    }
    /// Construct a function value using a native function.
    fn function_value(&self, function: FunctionValue<'ctx>) -> Self::Value {
        let func_name = function.get_name().to_str().unwrap();
        let func_name_ptr = self.native_global_string(func_name, func_name).into();
        let lambda_fn_ptr = self.builder.build_bitcast(
            function.as_global_value().as_pointer_value(),
            self.context.i64_type().ptr_type(AddressSpace::default()),
            "",
        );
        self.build_call(
            &ApiFunc::kclvm_value_Function_using_ptr.name(),
            &[self.current_runtime_ctx_ptr(), lambda_fn_ptr, func_name_ptr],
        )
    }
    /// Construct a closure function value with the closure variable.
    fn closure_value(&self, function: FunctionValue<'ctx>, closure: Self::Value) -> Self::Value {
        let func_name = function.get_name().to_str().unwrap();
        let func_name_ptr = self.native_global_string(func_name, func_name).into();
        // Convert the function to a i64 pointer to store it into the function value.
        let fn_ptr = self.builder.build_bitcast(
            function.as_global_value().as_pointer_value(),
            self.context.i64_type().ptr_type(AddressSpace::default()),
            "",
        );
        self.build_call(
            &ApiFunc::kclvm_value_Function.name(),
            &[
                self.current_runtime_ctx_ptr(),
                fn_ptr,
                closure,
                func_name_ptr,
                self.native_i8_zero().into(),
            ],
        )
    }
    /// Construct a schema function value using native functions.
    fn struct_function_value(
        &self,
        functions: &[FunctionValue<'ctx>],
        attr_functions: &HashMap<String, Vec<FunctionValue<'ctx>>>,
        runtime_type: &str,
    ) -> Self::Value {
        if functions.is_empty() {
            return self.none_value();
        }
        // Convert the function to a i64 pointer to store it into the function value.
        let schema_body_fn_ptr = self.builder.build_bitcast(
            functions[0].as_global_value().as_pointer_value(),
            self.context.i64_type().ptr_type(AddressSpace::default()),
            "",
        );
        // Convert the function to a i64 pointer to store it into the function value.
        let check_block_fn_ptr = if functions.len() > 1 {
            self.builder.build_bitcast(
                functions[1].as_global_value().as_pointer_value(),
                self.context.i64_type().ptr_type(AddressSpace::default()),
                "",
            )
        } else {
            self.context
                .i64_type()
                .ptr_type(AddressSpace::default())
                .const_zero()
                .into()
        };
        let runtime_type_native_str = self.native_global_string_value(runtime_type);
        let attr_map = self.dict_value();
        for attr in attr_functions.keys() {
            self.dict_insert_override_item(attr_map, attr, self.undefined_value())
        }
        self.builder
            .build_call(
                self.lookup_function(&ApiFunc::kclvm_value_schema_function.name()),
                &[
                    self.current_runtime_ctx_ptr().into(),
                    schema_body_fn_ptr.into(),
                    check_block_fn_ptr.into(),
                    attr_map.into(),
                    runtime_type_native_str.into(),
                ],
                runtime_type,
            )
            .try_as_basic_value()
            .left()
            .expect(kcl_error::FUNCTION_RETURN_VALUE_NOT_FOUND_MSG)
    }
    /// Construct a builtin function value using the function name.
    fn builtin_function_value(&self, function_name: &str) -> Self::Value {
        let mut function = self
            .module
            .get_function(function_name)
            .unwrap_or_else(|| panic!("global function {} not found", function_name));
        if self.no_link {
            let pkgpath = self.current_pkgpath();
            let modules = self.modules.borrow_mut();
            let msg = format!("pkgpath {} is not found", pkgpath);
            let module = &modules.get(&pkgpath).expect(&msg).borrow_mut().inner;
            let fn_type = function.get_type();
            function = module.add_function(function_name, fn_type, Some(Linkage::External));
        }
        self.function_value(function)
    }
    /// Get a global value pointer named `name`.
    fn global_value_ptr(&self, name: &str) -> Self::Value {
        let tpe = self.value_ptr_type();
        // Builtin function value is a global one
        let global_var = if self.no_link {
            let pkgpath = self.current_pkgpath();
            let msg = format!("pkgpath {} is not found", pkgpath);
            let modules = self.modules.borrow_mut();
            let module = &modules.get(&pkgpath).expect(&msg).borrow_mut().inner;
            module.add_global(tpe, Some(AddressSpace::default()), name)
        } else {
            self.module
                .add_global(tpe, Some(AddressSpace::default()), name)
        };
        global_var.set_alignment(GLOBAL_VAL_ALIGNMENT);
        global_var.set_initializer(&tpe.const_zero());
        global_var.as_pointer_value().into()
    }
    /// Get the global runtime context pointer.
    fn current_runtime_ctx_ptr(&self) -> Self::Value {
        self.builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap()
            .get_first_param()
            .expect(kcl_error::CONTEXT_VAR_NOT_FOUND_MSG)
    }
}

impl<'ctx> ValueCalculationMethods for LLVMCodeGenContext<'ctx> {
    /// lhs + rhs
    fn add(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_op_add.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs - rhs
    fn sub(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_op_sub.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs * rhs
    fn mul(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_op_mul.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs / rhs
    fn div(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_op_div.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs // rhs
    fn floor_div(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_op_floor_div.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs % rhs
    fn r#mod(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_op_mod.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs ** rhs
    fn pow(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_op_pow.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs << rhs
    fn bit_lshift(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_op_bit_lshift.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs >> rhs
    fn bit_rshift(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_op_bit_rshift.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs & rhs
    fn bit_and(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_op_bit_and.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs | rhs
    fn bit_or(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_op_bit_or.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs ^ rhs
    fn bit_xor(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_op_bit_xor.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs and rhs
    fn logic_and(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_logic_and.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs or rhs
    fn logic_or(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_logic_or.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs == rhs
    fn cmp_equal_to(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_cmp_equal_to.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs != rhs
    fn cmp_not_equal_to(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_cmp_not_equal_to.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs > rhs
    fn cmp_greater_than(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_cmp_greater_than.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs >= rhs
    fn cmp_greater_than_or_equal(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_cmp_greater_than_or_equal.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs < rhs
    fn cmp_less_than(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_cmp_less_than.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs <= rhs
    fn cmp_less_than_or_equal(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_cmp_less_than_or_equal.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs as rhs
    fn r#as(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_as.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs is rhs
    fn is(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_is.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs is not rhs
    fn is_not(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_is_not.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs in rhs
    fn r#in(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_in.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
    /// lhs not in rhs
    fn not_in(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_not_in.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        )
    }
}

impl<'ctx> DerivedValueCalculationMethods for LLVMCodeGenContext<'ctx> {
    /// Value subscript a[b]
    #[inline]
    fn value_subscript(&self, value: Self::Value, item: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_subscr.name(),
            &[self.current_runtime_ctx_ptr(), value, item],
        )
    }
    /// Value is truth function, return i1 value.
    fn value_is_truthy(&self, value: Self::Value) -> Self::Value {
        let is_truth = self
            .build_call(&ApiFunc::kclvm_value_is_truthy.name(), &[value])
            .into_int_value();
        self.builder
            .build_int_compare(IntPredicate::NE, is_truth, self.native_i8_zero(), "")
            .into()
    }
    /// Value deep copy
    #[inline]
    fn value_deep_copy(&self, value: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_deep_copy.name(),
            &[self.current_runtime_ctx_ptr(), value],
        )
    }
    /// value_union unions two collection elements.
    #[inline]
    fn value_union(&self, lhs: Self::Value, rhs: Self::Value) {
        self.build_void_call(
            &ApiFunc::kclvm_value_union.name(),
            &[self.current_runtime_ctx_ptr(), lhs, rhs],
        );
    }
    // List get the item using the index.
    #[inline]
    fn list_get(&self, list: Self::Value, index: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_list_get.name(),
            &[self.current_runtime_ctx_ptr(), list, index],
        )
    }
    // List set the item using the index.
    #[inline]
    fn list_set(&self, list: Self::Value, index: Self::Value, value: Self::Value) {
        self.build_void_call(&ApiFunc::kclvm_list_set.name(), &[list, index, value])
    }
    // List slice.
    #[inline]
    fn list_slice(
        &self,
        list: Self::Value,
        start: Self::Value,
        stop: Self::Value,
        step: Self::Value,
    ) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_value_slice.name(),
            &[self.current_runtime_ctx_ptr(), list, start, stop, step],
        )
    }
    /// Append a item into the list.
    #[inline]
    fn list_append(&self, list: Self::Value, item: Self::Value) {
        self.build_void_call(&ApiFunc::kclvm_list_append.name(), &[list, item])
    }
    /// Append a list item and unpack it into the list.
    #[inline]
    fn list_append_unpack(&self, list: Self::Value, item: Self::Value) {
        self.build_void_call(&ApiFunc::kclvm_list_append_unpack.name(), &[list, item]);
    }
    /// Runtime list value pop
    #[inline]
    fn list_pop(&self, list: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_list_pop.name(),
            &[self.current_runtime_ctx_ptr(), list],
        )
    }
    /// Runtime list pop the first value
    #[inline]
    fn list_pop_first(&self, list: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_list_pop_first.name(),
            &[self.current_runtime_ctx_ptr(), list],
        )
    }
    /// List clear value.
    #[inline]
    fn list_clear(&self, list: Self::Value) {
        self.build_void_call(&ApiFunc::kclvm_list_clear.name(), &[list])
    }
    /// Return number of occurrences of the list value.
    #[inline]
    fn list_count(&self, list: Self::Value, item: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_list_count.name(),
            &[self.current_runtime_ctx_ptr(), list, item],
        )
    }
    /// Return first index of the list value. Panic if the value is not present.
    #[inline]
    fn list_find(&self, list: Self::Value, item: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_list_find.name(),
            &[self.current_runtime_ctx_ptr(), list, item],
        )
    }
    /// Insert object before index of the list value.
    #[inline]
    fn list_insert(&self, list: Self::Value, index: Self::Value, value: Self::Value) {
        self.build_void_call(&ApiFunc::kclvm_list_insert.name(), &[list, index, value])
    }
    /// List length.
    #[inline]
    fn list_len(&self, list: Self::Value) -> Self::Value {
        self.build_call(&ApiFunc::kclvm_list_len.name(), &[list])
    }
    /// Dict get the value of the key.
    #[inline]
    fn dict_get(&self, dict: Self::Value, key: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_dict_get_value.name(),
            &[self.current_runtime_ctx_ptr(), dict, key],
        )
    }
    /// Dict set the value of the key.
    #[inline]
    fn dict_set(&self, dict: Self::Value, key: Self::Value, value: Self::Value) {
        self.build_void_call(
            &ApiFunc::kclvm_dict_set_value.name(),
            &[self.current_runtime_ctx_ptr(), dict, key, value],
        )
    }
    /// Return all dict keys.
    #[inline]
    fn dict_keys(&self, dict: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_dict_keys.name(),
            &[self.current_runtime_ctx_ptr(), dict],
        )
    }
    /// Return all dict values.
    #[inline]
    fn dict_values(&self, dict: Self::Value) -> Self::Value {
        self.build_call(
            &ApiFunc::kclvm_dict_values.name(),
            &[self.current_runtime_ctx_ptr(), dict],
        )
    }
    /// Dict clear value.
    #[inline]
    fn dict_clear(&self, dict: Self::Value) {
        self.build_void_call(
            &ApiFunc::kclvm_dict_insert_value.name(),
            &[self.current_runtime_ctx_ptr(), dict],
        )
    }
    /// Dict pop the value of the key.
    #[inline]
    fn dict_pop(&self, dict: Self::Value, key: Self::Value) -> Self::Value {
        self.build_call(&ApiFunc::kclvm_dict_remove.name(), &[dict, key])
    }
    /// Dict length.
    #[inline]
    fn dict_len(&self, dict: Self::Value) -> Self::Value {
        self.build_call(&ApiFunc::kclvm_dict_len.name(), &[dict])
    }
    /// Insert a dict entry including key, value, op and insert_index into the dict,
    /// and the type of key is `&str`
    #[inline]
    fn dict_insert(
        &self,
        dict: Self::Value,
        key: &str,
        value: Self::Value,
        op: i32,
        insert_index: i32,
    ) {
        let name = self.native_global_string(key, "").into();
        let op = self.native_int_value(op);
        let insert_index = self.native_int_value(insert_index);
        self.build_void_call(
            &ApiFunc::kclvm_dict_insert.name(),
            &[
                self.current_runtime_ctx_ptr(),
                dict,
                name,
                value,
                op,
                insert_index,
            ],
        );
    }

    /// Insert a dict entry including key, value, op and insert_index into the dict.
    /// and the type of key is `Self::Value`
    #[inline]
    fn dict_insert_with_key_value(
        &self,
        dict: Self::Value,
        key: Self::Value,
        value: Self::Value,
        op: i32,
        insert_index: i32,
    ) {
        let op = self.native_int_value(op);
        let insert_index = self.native_int_value(insert_index);
        self.build_void_call(
            &ApiFunc::kclvm_dict_insert_value.name(),
            &[
                self.current_runtime_ctx_ptr(),
                dict,
                key,
                value,
                op,
                insert_index,
            ],
        );
    }
}

impl<'ctx> ValueCodeGen for LLVMCodeGenContext<'ctx> {}

/* Type methods */

impl<'ctx> BaseTypeMethods for LLVMCodeGenContext<'ctx> {
    /// Native i8 type
    fn i8_type(&self) -> Self::Type {
        self.context.i8_type().into()
    }
    /// Native i16 type
    fn i16_type(&self) -> Self::Type {
        self.context.i16_type().into()
    }
    /// Native i32 type
    fn i32_type(&self) -> Self::Type {
        self.context.i32_type().into()
    }
    /// Native i64 type
    fn i64_type(&self) -> Self::Type {
        self.context.i64_type().into()
    }
    /// Native i128 type
    fn i128_type(&self) -> Self::Type {
        self.context.i128_type().into()
    }
    /// Native f32 type
    fn f32_type(&self) -> Self::Type {
        self.context.f32_type().into()
    }
    /// Native f64 type
    fn f64_type(&self) -> Self::Type {
        self.context.f64_type().into()
    }
    /// Native struct type.
    #[inline]
    fn struct_type(&self, els: &[Self::Type], packed: bool) -> Self::Type {
        self.context.struct_type(els, packed).into()
    }
    /// Native pointer type of `ty`.
    #[inline]
    fn ptr_type_to(&self, ty: Self::Type) -> Self::Type {
        self.ptr_type_to_ext(ty, crate::codegen::abi::AddressSpace::DATA)
    }
    /// Native pointer type of `ty` with the address space.
    #[inline]
    fn ptr_type_to_ext(
        &self,
        ty: Self::Type,
        address_space: crate::codegen::abi::AddressSpace,
    ) -> Self::Type {
        let address_space =
            AddressSpace::try_from(address_space.0).expect(kcl_error::INTERNAL_ERROR_MSG);
        let ptr_type = match ty {
            BasicTypeEnum::ArrayType(a) => a.ptr_type(address_space),
            BasicTypeEnum::FloatType(f) => f.ptr_type(address_space),
            BasicTypeEnum::IntType(i) => i.ptr_type(address_space),
            BasicTypeEnum::PointerType(p) => p.ptr_type(address_space),
            BasicTypeEnum::StructType(s) => s.ptr_type(address_space),
            BasicTypeEnum::VectorType(v) => v.ptr_type(address_space),
        };
        ptr_type.into()
    }
    /// Native array element type.
    #[inline]
    fn element_type(&self, ty: Self::Type) -> Self::Type {
        match ty {
            BasicTypeEnum::ArrayType(a) => a.get_element_type(),
            BasicTypeEnum::VectorType(v) => v.get_element_type(),
            other => panic!("element_type called on unsupported type {:?}", other),
        }
    }
    /// Returns the number of elements in `self` if it is a LLVM vector type.
    #[inline]
    fn vector_length(&self, ty: Self::Type) -> usize {
        ty.into_vector_type().get_size() as usize
    }
    /// Retrieves the bit width of the float type `self`.
    #[inline]
    fn float_width(&self, ty: Self::Type) -> usize {
        let ty_str = format!("{:?}", ty.into_float_type());
        for (float_ty, float_width) in FLOAT_TYPE_WIDTH_MAPPING.into_iter() {
            if ty_str.contains(float_ty) {
                return *float_width;
            }
        }
        panic!("float_width called on unsupported type {:?}", ty);
    }
    /// Retrieves the bit width of the integer type `self`.
    #[inline]
    fn int_width(&self, ty: Self::Type) -> usize {
        ty.into_int_type().get_bit_width() as usize
    }
    /// Get the value type.
    #[inline]
    fn val_type(&self, v: Self::Value) -> Self::Type {
        v.get_type()
    }
    /// Native function type
    #[inline]
    fn function_let(&self, args: &[Self::Type], ret: Self::Type) -> Self::FunctionLet {
        let args: Vec<BasicMetadataTypeEnum> = args.iter().map(|arg| (*arg).into()).collect();
        ret.fn_type(&args, false)
    }
}

impl<'ctx> DerivedTypeMethods for LLVMCodeGenContext<'ctx> {
    /// Lookup type by the type name.
    #[inline]
    fn get_intrinsic_type(&self, name: &str) -> Self::Type {
        self.module
            .get_struct_type(name)
            .expect(kcl_error::VALUE_TYPE_NOT_FOUND_MSG)
            .into()
    }
}

impl<'ctx> TypeCodeGen for LLVMCodeGenContext<'ctx> {}

impl<'ctx> ProgramCodeGen for LLVMCodeGenContext<'ctx> {
    /// Current package path
    fn current_pkgpath(&self) -> String {
        self.pkgpath_stack
            .borrow_mut()
            .last()
            .expect(kcl_error::INTERNAL_ERROR_MSG)
            .to_string()
    }

    /// Current filename
    fn current_filename(&self) -> String {
        self.filename_stack
            .borrow_mut()
            .last()
            .expect(kcl_error::INTERNAL_ERROR_MSG)
            .to_string()
    }
    /// Init a scope named `pkgpath` with all builtin functions
    fn init_scope(&self, pkgpath: &str) {
        {
            let mut pkg_scopes = self.pkg_scopes.borrow_mut();
            if pkg_scopes.contains_key(pkgpath) {
                return;
            }
            let scopes = vec![Rc::new(Scope::default())];
            pkg_scopes.insert(String::from(pkgpath), scopes);
        }
        let msg = format!("pkgpath {} is not found", pkgpath);
        // Init all global types including schema and rule
        let module_list: &Vec<ast::Module> = if self.program.pkgs.contains_key(pkgpath) {
            self.program.pkgs.get(pkgpath).expect(&msg)
        } else if pkgpath.starts_with(kclvm_runtime::PKG_PATH_PREFIX)
            && self.program.pkgs.contains_key(&pkgpath[1..])
        {
            self.program
                .pkgs
                .get(&pkgpath[1..])
                .expect(kcl_error::INTERNAL_ERROR_MSG)
        } else {
            panic!("pkgpath {} not found", pkgpath);
        };
        for module in module_list {
            for stmt in &module.body {
                let name = match &stmt.node {
                    ast::Stmt::Schema(schema_stmt) => schema_stmt.name.node.clone(),
                    ast::Stmt::Rule(rule_stmt) => rule_stmt.name.node.clone(),
                    _ => "".to_string(),
                };
                if !name.is_empty() {
                    let name = name.as_str();
                    let var_name = format!("${}.${}", pkgpath_without_prefix!(pkgpath), name);
                    let global_var_ptr = self.new_global_kcl_value_ptr(&var_name);
                    self.add_variable(name, global_var_ptr);
                }
            }
        }
        // Init all builtin functions
        for symbol in builtin::BUILTIN_FUNCTION_NAMES {
            let function_name =
                format!("{}_{}", builtin::KCL_BUILTIN_FUNCTION_MANGLE_PREFIX, symbol);
            let function_value = self.builtin_function_value(function_name.as_str());
            let builtin_function_name = format!(
                "{}_{}_{}",
                builtin::BUILTIN_FUNCTION_PREFIX,
                pkgpath_without_prefix!(pkgpath),
                function_name
            );
            let global_var_ptr = self.new_global_kcl_value_ptr(&builtin_function_name);
            self.builder.build_store(global_var_ptr, function_value);
            self.add_variable(symbol, global_var_ptr);
        }
        self.enter_scope();
    }

    /// Get the scope level
    fn scope_level(&self) -> usize {
        let current_pkgpath = self.current_pkgpath();
        let pkg_scopes = self.pkg_scopes.borrow_mut();
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get(&current_pkgpath).expect(&msg);
        // Sub the builtin global scope
        scopes.len() - 1
    }

    /// Enter scope
    fn enter_scope(&self) {
        let current_pkgpath = self.current_pkgpath();
        let mut pkg_scopes = self.pkg_scopes.borrow_mut();
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        let scope = Rc::new(Scope::default());
        scopes.push(scope);
    }

    /// Leave scope
    fn leave_scope(&self) {
        let current_pkgpath = self.current_pkgpath();
        let mut pkg_scopes = self.pkg_scopes.borrow_mut();
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        scopes.pop();
    }
}

impl<'ctx> CodeGenContext for LLVMCodeGenContext<'ctx> {
    /// Generate LLVM IR of ast module.
    fn emit(&self, opt: &EmitOptions) -> Result<(), Box<dyn Error>> {
        self.emit_code(opt)
    }
}

impl<'ctx> LLVMCodeGenContext<'ctx> {
    /// New aa LLVMCodeGenContext using the LLVM Context and AST Program
    pub fn new(
        context: &'ctx Context,
        module: Module<'ctx>,
        program: &'ctx ast::Program,
        import_names: IndexMap<String, IndexMap<String, String>>,
        no_link: bool,
        workdir: String,
    ) -> LLVMCodeGenContext<'ctx> {
        LLVMCodeGenContext {
            context,
            module,
            builder: context.create_builder(),
            program,
            pkg_scopes: RefCell::new(HashMap::new()),
            functions: RefCell::new(vec![]),
            imported: RefCell::new(HashSet::new()),
            local_vars: RefCell::new(HashSet::new()),
            schema_stack: RefCell::new(vec![]),
            // 1 denotes the top global main function lambda and 0 denotes the builtin scope.
            // Any user-defined lambda scope greater than 1.
            lambda_stack: RefCell::new(vec![GLOBAL_LEVEL]),
            schema_expr_stack: RefCell::new(vec![]),
            pkgpath_stack: RefCell::new(vec![String::from(MAIN_PKG_PATH)]),
            filename_stack: RefCell::new(vec![String::from("")]),
            target_vars: RefCell::new(vec![String::from("")]),
            global_strings: RefCell::new(IndexMap::default()),
            global_vars: RefCell::new(IndexMap::default()),
            current_filename: RefCell::new(String::new()),
            current_line: RefCell::new(0),
            handler: RefCell::new(Handler::default()),
            backtrack_meta: RefCell::new(None),
            import_names,
            no_link,
            debug: false,
            modules: RefCell::new(HashMap::new()),
            workdir,
        }
    }

    /// Generate LLVM IR of ast module.
    pub(crate) fn emit_code(
        self: &LLVMCodeGenContext<'ctx>,
        opt: &EmitOptions,
    ) -> Result<(), Box<dyn Error>> {
        let tpe = self.value_ptr_type().into_pointer_type();
        let void_type = self.context.void_type();
        let context_ptr_type = self.context_ptr_type();
        let fn_type = tpe.fn_type(&[context_ptr_type.into()], false);
        let void_fn_type = void_type.fn_type(&[context_ptr_type.into()], false);
        let has_main_pkg = self.program.pkgs.contains_key(MAIN_PKG_PATH);
        let function = if self.no_link {
            let mut modules = self.modules.borrow_mut();
            let (pkgpath, function_name) = if has_main_pkg {
                (MAIN_PKG_PATH.to_string(), MODULE_NAME.to_string())
            } else {
                assert!(self.program.pkgs.len() == 1);
                let pkgpath = format!(
                    "{}{}",
                    kclvm_runtime::PKG_PATH_PREFIX,
                    self.program
                        .pkgs
                        .keys()
                        .next()
                        .expect(kcl_error::INTERNAL_ERROR_MSG)
                );
                (
                    pkgpath.clone(),
                    format!(
                        "${}.{}",
                        pkgpath_without_prefix!(pkgpath),
                        PKG_INIT_FUNCTION_SUFFIX
                    ),
                )
            };
            let module = self.context.create_module(pkgpath.as_str());
            let function = module.add_function(
                // Function name
                function_name.as_str(),
                // Function type
                if has_main_pkg { fn_type } else { void_fn_type },
                None,
            );
            modules.insert(
                pkgpath.to_string(),
                RefCell::new(self.create_debug_module(module)),
            );
            function
        } else {
            self.module.add_function(
                // Function name
                MODULE_NAME,
                // Function type
                fn_type,
                None,
            )
        };
        self.push_function(function);
        // Add a block named entry into the function
        let basic_block = self.append_block(ENTRY_NAME);
        // Set position to the basic block
        self.builder.position_at_end(basic_block);
        // Get the runtime context
        let ctx_value = function
            .get_first_param()
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        if self.no_link && !has_main_pkg {
            for pkgpath in self.program.pkgs.keys() {
                let pkgpath = format!("{}{}", kclvm_runtime::PKG_PATH_PREFIX, pkgpath);
                self.pkgpath_stack.borrow_mut().push(pkgpath.clone());
            }
        }
        // Set the kcl module path to the runtime context
        self.build_void_call(
            &ApiFunc::kclvm_context_set_kcl_modpath.name(),
            &[
                self.current_runtime_ctx_ptr(),
                self.native_global_string_value(&self.program.root),
            ],
        );
        // Set the kcl workdir to the runtime context
        self.build_void_call(
            &ApiFunc::kclvm_context_set_kcl_workdir.name(),
            &[
                self.current_runtime_ctx_ptr(),
                self.native_global_string_value(&self.workdir),
            ],
        );
        if !self.import_names.is_empty() {
            let import_names = self.dict_value();
            for (k, v) in &self.import_names {
                let map = self.dict_value();
                for (pkgname, pkgpath) in v {
                    self.dict_insert_override_item(
                        map,
                        pkgname,
                        self.string_value(&format!("@{}", pkgpath)),
                    );
                }
                self.dict_insert_override_item(import_names, k, map);
            }
            self.build_void_call(
                &ApiFunc::kclvm_context_set_import_names.name(),
                &[ctx_value, import_names],
            );
        }
        // Main package
        if self.no_link && !has_main_pkg {
            // When compiling a pkgpath separately, only one pkgpath is required in the AST Program
            assert!(self.program.pkgs.len() == 1);
            // pkgs may not contains main pkg in no link mode
            for (pkgpath, modules) in &self.program.pkgs {
                let pkgpath = format!("{}{}", kclvm_runtime::PKG_PATH_PREFIX, pkgpath);
                self.pkgpath_stack.borrow_mut().push(pkgpath.clone());
                // Init all builtin functions.
                self.init_scope(pkgpath.as_str());
                self.compile_ast_modules(modules);
            }
            self.ret_void();
        } else {
            // Init scope and all builtin functions
            self.init_scope(MAIN_PKG_PATH);
            let main_pkg_modules = self
                .program
                .pkgs
                .get(MAIN_PKG_PATH)
                .expect(kcl_error::INTERNAL_ERROR_MSG);
            self.compile_ast_modules(main_pkg_modules);
            // Get the JSON string including all global variables
            let json_str_value = self.globals_to_json_str();
            // Build a return in the current block
            self.pop_function();
            self.builder
                .build_return(Some(&json_str_value.into_pointer_value()));
        }
        if let Some(path_str) = &opt.emit_path {
            let path = std::path::Path::new(&path_str);
            if opt.no_link {
                let modules = self.modules.borrow_mut();
                for (index, (_, module)) in modules.iter().enumerate() {
                    let path = if modules.len() == 1 {
                        format!("{}{}", path_str, OBJECT_FILE_SUFFIX)
                    } else {
                        format!("{}_{}{}", path_str, index, OBJECT_FILE_SUFFIX)
                    };
                    let path = std::path::Path::new(&path);
                    // Build LLVM module to a `.o` object file.
                    self.build_object_file(&module.borrow().inner, path)?;
                }
            } else {
                // Build LLVM module to a `.o` object file.
                self.build_object_file(&self.module, path)?;
            }
        }
        Ok(())
    }

    /// Build LLVM module to a `.o` object file.
    ///
    /// TODO: WASM and cross platform build.
    fn build_object_file(
        self: &LLVMCodeGenContext<'ctx>,
        module: &Module,
        path: &Path,
    ) -> Result<(), LLVMString> {
        let triple = inkwell::targets::TargetMachine::get_default_triple();
        let target = inkwell::targets::Target::from_triple(&triple)?;
        // Convert LLVM module to ll file.
        module.print_to_file(path)?;
        let buf = MemoryBuffer::create_from_file(path)?;
        let module = self.context.create_module_from_ir(buf)?;
        // Read ll file and use target machine to generate native object file.
        let target_machine = target
            .create_target_machine(
                &triple,
                "",
                "",
                // We do not enable any optimization, so that
                // the sum of compile time and run time is as small as possible
                inkwell::OptimizationLevel::None,
                RelocMode::PIC,
                CodeModel::Default,
            )
            .expect(kcl_error::CODE_GEN_ERROR_MSG);
        target_machine.write_to_file(&module, FileType::Object, path)
    }
}

impl<'ctx> LLVMCodeGenContext<'ctx> {
    /// Get compiler default ok result
    #[inline]
    pub fn ok_result(&self) -> CompileResult<'ctx> {
        let i32_type = self.context.i32_type();
        Ok(i32_type.const_int(0u64, false).into())
    }

    /// Build a void function call
    #[inline]
    pub fn build_void_call(&self, name: &str, args: &[BasicValueEnum]) {
        let args: Vec<BasicMetadataValueEnum> = args.iter().map(|arg| (*arg).into()).collect();
        self.builder
            .build_call(self.lookup_function(name), &args, "");
    }

    /// Build a function call with the return value
    #[inline]
    pub fn build_call(&self, name: &str, args: &[BasicValueEnum<'ctx>]) -> BasicValueEnum<'ctx> {
        let args: Vec<BasicMetadataValueEnum> = args.iter().map(|arg| (*arg).into()).collect();
        self.builder
            .build_call(self.lookup_function(name), &args, "")
            .try_as_basic_value()
            .left()
            .expect(kcl_error::FUNCTION_RETURN_VALUE_NOT_FOUND_MSG)
    }

    /// Creates global string in the llvm module with initializer
    pub fn native_global_string(&self, value: &str, name: &str) -> PointerValue<'ctx> {
        let mut global_string_maps = self.global_strings.borrow_mut();
        let pkgpath = self.current_pkgpath();
        let str_name = format!("${}_{}_str", pkgpath_without_prefix!(pkgpath), name);
        if !global_string_maps.contains_key(&pkgpath) {
            global_string_maps.insert(pkgpath.clone(), IndexMap::default());
        }
        let msg = format!("pkgpath {} is not found", pkgpath);
        let global_strings = global_string_maps.get_mut(&pkgpath).expect(&msg);
        if let Some(ptr) = global_strings.get(value) {
            *ptr
        } else {
            let gv = unsafe { self.builder.build_global_string(value, &str_name) };
            let ptr = self
                .ptr_cast(
                    gv.as_pointer_value().into(),
                    self.ptr_type_to(self.i8_type()),
                )
                .into_pointer_value();
            global_strings.insert(value.to_string(), ptr);
            ptr
        }
    }

    /// Creates global string value in the llvm module with initializer
    pub fn native_global_string_value(&self, value: &str) -> BasicValueEnum<'ctx> {
        let pkgpath = self.current_pkgpath();
        let str_name = format!("${}_str", pkgpath_without_prefix!(pkgpath));
        self.native_global_string(value, &str_name).into()
    }

    /// Get LLVM i8 zero value
    pub fn native_i8_zero(&self) -> IntValue<'ctx> {
        let i8_type = self.context.i8_type();
        i8_type.const_int(0u64, false)
    }

    /// Get LLVM i8 zero value
    pub fn native_i8(&self, v: i8) -> IntValue<'ctx> {
        let i8_type = self.context.i8_type();
        i8_type.const_int(v as u64, false)
    }

    /// Construct a LLVM int value using i32
    pub fn native_int_value(&self, v: i32) -> BasicValueEnum<'ctx> {
        let i32_type = self.context.i32_type();
        i32_type.const_int(v as u64, false).into()
    }

    /// Construct a global value pointer named `name`
    pub fn new_global_kcl_value_ptr(&self, name: &str) -> PointerValue<'ctx> {
        let tpe = self.value_ptr_type();
        // Builtin function value is a global one
        let global_var = if self.no_link {
            let pkgpath = self.current_pkgpath();
            let msg = format!("pkgpath {} is not found", pkgpath);
            let modules = self.modules.borrow_mut();
            let module = &modules.get(&pkgpath).expect(&msg).borrow_mut().inner;
            module.add_global(tpe, Some(AddressSpace::default()), name)
        } else {
            self.module
                .add_global(tpe, Some(AddressSpace::default()), name)
        };
        global_var.set_alignment(GLOBAL_VAL_ALIGNMENT);
        global_var.set_initializer(&tpe.const_zero());
        global_var.as_pointer_value()
    }

    /// Append a scalar value into the scope.
    pub fn add_scalar(&self, scalar: BasicValueEnum<'ctx>, is_schema: bool) {
        let current_pkgpath = self.current_pkgpath();
        let mut pkg_scopes = self.pkg_scopes.borrow_mut();
        let scopes = pkg_scopes
            .get_mut(&current_pkgpath)
            .unwrap_or_else(|| panic!("pkgpath {} is not found", current_pkgpath));
        if let Some(last) = scopes.last_mut() {
            let mut scalars = last.scalars.borrow_mut();
            // TODO: To avoid conflicts, only the last schema scalar expressions are allowed.
            let mut schema_scalar_idx = last.schema_scalar_idx.borrow_mut();
            if is_schema {
                // Remove the last schema scalar.
                if *schema_scalar_idx < scalars.len() {
                    scalars.remove(*schema_scalar_idx);
                }
                // Override the last schema scalar.
                scalars.push(scalar);
                *schema_scalar_idx = scalars.len() - 1;
            } else {
                scalars.push(scalar);
            }
        }
    }

    /// Append a variable into the scope
    pub fn add_variable(&self, name: &str, pointer: PointerValue<'ctx>) {
        let current_pkgpath = self.current_pkgpath();
        let mut pkg_scopes = self.pkg_scopes.borrow_mut();
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        if let Some(last) = scopes.last_mut() {
            let mut variables = last.variables.borrow_mut();
            if !variables.contains_key(name) {
                variables.insert(name.to_string(), pointer);
            }
        }
    }

    /// Store the argument named `name` in the current scope.
    pub(crate) fn store_argument_in_current_scope(&self, name: &str) {
        // Find argument name in the scope
        let current_pkgpath = self.current_pkgpath();
        let mut pkg_scopes = self.pkg_scopes.borrow_mut();
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        let index = scopes.len() - 1;
        let mut arguments_mut = scopes[index].arguments.borrow_mut();
        arguments_mut.insert(name.to_string());
    }

    /// Store the variable named `name` with `value` from the current scope, return false when not found
    pub fn store_variable_in_current_scope(&self, name: &str, value: BasicValueEnum<'ctx>) -> bool {
        // Find argument name in the scope
        let current_pkgpath = self.current_pkgpath();
        let pkg_scopes = self.pkg_scopes.borrow();
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get(&current_pkgpath).expect(&msg);
        let index = scopes.len() - 1;
        let variables = scopes[index].variables.borrow();
        if let Some(var) = variables.get(&name.to_string()) {
            self.builder.build_store(*var, value);
            return true;
        }
        false
    }

    /// Store the variable named `name` with `value` from the scope, return false when not found
    pub fn store_variable(&self, name: &str, value: BasicValueEnum<'ctx>) -> bool {
        // Find argument name in the scope
        let current_pkgpath = self.current_pkgpath();
        let pkg_scopes = self.pkg_scopes.borrow();
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get(&current_pkgpath).expect(&msg);
        for i in 0..scopes.len() {
            let index = scopes.len() - i - 1;
            let variables = scopes[index].variables.borrow();
            if let Some(var) = variables.get(&name.to_string()) {
                self.builder.build_store(*var, value);
                return true;
            }
        }
        false
    }

    /// Resolve variable in scope, return false when not found.
    #[inline]
    pub fn resolve_variable(&self, name: &str) -> bool {
        self.resolve_variable_level(name).is_some()
    }

    /// Resolve variable level in scope, return None when not found.
    pub fn resolve_variable_level(&self, name: &str) -> Option<usize> {
        // Find argument name in the scope
        let current_pkgpath = self.current_pkgpath();
        let pkg_scopes = self.pkg_scopes.borrow();
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get(&current_pkgpath).expect(&msg);
        let mut level = None;
        for i in 0..scopes.len() {
            let index = scopes.len() - i - 1;
            let variables = scopes[index].variables.borrow();
            let arguments = scopes[index].arguments.borrow();
            if variables.get(name).is_some() {
                level = Some(index);
                break;
            }
            if arguments.contains(name) {
                level = Some(index);
                break;
            }
        }
        level
    }

    /// Append a variable or update the existed local variable.
    pub fn add_or_update_local_variable(&self, name: &str, value: BasicValueEnum<'ctx>) {
        let current_pkgpath = self.current_pkgpath();
        let mut pkg_scopes = self.pkg_scopes.borrow_mut();
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        let mut existed = false;
        // Query the variable in all scopes.
        for i in 0..scopes.len() {
            let index = scopes.len() - i - 1;
            let variables_mut = scopes[index].variables.borrow_mut();
            match variables_mut.get(&name.to_string()) {
                // If the local variable is found, store the new value for the variable.
                // We cannot update rule/lambda/schema arguments because they are read-only.
                Some(ptr)
                    if index > GLOBAL_LEVEL
                        && !self.local_vars.borrow().contains(name)
                        && !scopes[index].arguments.borrow().contains(name) =>
                {
                    self.builder.build_store(*ptr, value);
                    existed = true;
                }
                _ => {}
            }
        }
        // If not found, alloc a new variable.
        if !existed {
            let ptr = self.builder.build_alloca(self.value_ptr_type(), name);
            self.builder.build_store(ptr, value);
            // Store the value for the variable and add the variable into the current scope.
            if let Some(last) = scopes.last_mut() {
                let mut variables = last.variables.borrow_mut();
                variables.insert(name.to_string(), ptr);
            }
        }
    }

    /// Append a variable or update the existed variable
    pub fn add_or_update_global_variable(&self, name: &str, value: BasicValueEnum<'ctx>) {
        // Find argument name in the scope
        let current_pkgpath = self.current_pkgpath();
        let mut pkg_scopes = self.pkg_scopes.borrow_mut();
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        let mut existed = false;
        if let Some(last) = scopes.last_mut() {
            let variables = last.variables.borrow();
            if let Some(var) = variables.get(&name.to_string()) {
                self.builder.build_store(*var, value);
                existed = true;
            }
        }
        if !existed {
            if let Some(last) = scopes.last_mut() {
                let mut variables = last.variables.borrow_mut();
                let pkgpath = self.current_pkgpath();
                let var_name = format!("${}.${}", pkgpath_without_prefix!(pkgpath), name);
                let pointer = self.new_global_kcl_value_ptr(&var_name);
                self.builder.build_store(pointer, value);
                if !variables.contains_key(name) {
                    variables.insert(name.to_string(), pointer);
                }
            }
        }
    }

    /// Get the variable value named `name` from the scope, return Err when not found
    pub fn get_variable(&self, name: &str) -> CompileResult<'ctx> {
        let current_pkgpath = self.current_pkgpath();
        self.get_variable_in_pkgpath(name, &current_pkgpath)
    }

    /// Get the variable value named `name` from the scope, return Err when not found
    pub fn get_variable_in_schema(&self, name: &str) -> CompileResult<'ctx> {
        let schema_value = self
            .get_variable(value::SCHEMA_SELF_NAME)
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        let cal_map = self
            .get_variable(value::SCHEMA_CAL_MAP)
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        let string_ptr_value = self.native_global_string(name, "").into();
        let cal_map_has_key = self
            .build_call(
                &ApiFunc::kclvm_dict_has_value.name(),
                &[cal_map, string_ptr_value],
            )
            .into_int_value();
        let schema_has_key = self
            .build_call(
                &ApiFunc::kclvm_dict_has_value.name(),
                &[schema_value, string_ptr_value],
            )
            .into_int_value();
        // has_key = cal_map_has_key or schema_has_key
        let has_key = self
            .builder
            .build_int_add(cal_map_has_key, schema_has_key, "");
        let has_key =
            self.builder
                .build_int_compare(IntPredicate::NE, has_key, self.native_i8_zero(), "");
        let then_block = self.append_block("");
        let else_block = self.append_block("");
        let end_block = self.append_block("");
        self.builder
            .build_conditional_branch(has_key, then_block, else_block);
        self.builder.position_at_end(then_block);
        let target_attr = self
            .target_vars
            .borrow()
            .last()
            .expect(kcl_error::INTERNAL_ERROR_MSG)
            .clone();
        let target_attr = self.native_global_string_value(&target_attr);
        let config_attr_value = {
            let config = self
                .get_variable(value::SCHEMA_CONFIG_NAME)
                .expect(kcl_error::INTERNAL_ERROR_MSG);
            let config_meta = self
                .get_variable(value::SCHEMA_CONFIG_META_NAME)
                .expect(kcl_error::INTERNAL_ERROR_MSG);
            let backtrack_level_map = self
                .get_variable(value::BACKTRACK_LEVEL_MAP)
                .expect(kcl_error::INTERNAL_ERROR_MSG);
            let backtrack_cache = self
                .get_variable(value::BACKTRACK_CACHE)
                .expect(kcl_error::INTERNAL_ERROR_MSG);
            let args = self
                .get_variable(value::SCHEMA_ARGS)
                .expect(kcl_error::INTERNAL_ERROR_MSG);
            let kwargs = self
                .get_variable(value::SCHEMA_KWARGS)
                .expect(kcl_error::INTERNAL_ERROR_MSG);
            self.build_call(
                &ApiFunc::kclvm_schema_get_value.name(),
                &[
                    self.current_runtime_ctx_ptr(),
                    schema_value,
                    string_ptr_value,
                    config,
                    config_meta,
                    cal_map,
                    target_attr,
                    backtrack_level_map,
                    backtrack_cache,
                    args,
                    kwargs,
                ],
            )
        };
        self.br(end_block);
        self.builder.position_at_end(else_block);
        let current_pkgpath = self.current_pkgpath();
        let result = self.get_variable_in_pkgpath(name, &current_pkgpath);
        let value = match result {
            Ok(v) => v,
            Err(_) => self.undefined_value(),
        };
        self.br(end_block);
        self.builder.position_at_end(end_block);
        let tpe = self.value_ptr_type();
        let phi = self.builder.build_phi(tpe, "");
        phi.add_incoming(&[(&value, else_block), (&config_attr_value, then_block)]);
        let value = phi.as_basic_value();
        Ok(value)
    }

    /// Get the variable value named `name` from the scope named `pkgpath`, return Err when not found
    pub fn get_variable_in_pkgpath(&self, name: &str, pkgpath: &str) -> CompileResult<'ctx> {
        let pkg_scopes = self.pkg_scopes.borrow();
        let pkgpath =
            if !pkgpath.starts_with(kclvm_runtime::PKG_PATH_PREFIX) && pkgpath != MAIN_PKG_PATH {
                format!("{}{}", kclvm_runtime::PKG_PATH_PREFIX, pkgpath)
            } else {
                pkgpath.to_string()
            };
        let mut result = Err(kcl_error::KCLError {
            message: format!("name '{}' is not defined", name),
            ty: kcl_error::KCLErrorType::Compile,
        });
        let is_in_schema = self.is_in_schema();
        // System module
        if builtin::STANDARD_SYSTEM_MODULE_NAMES_WITH_AT.contains(&pkgpath.as_str()) {
            let pkgpath = &pkgpath[1..];
            let mangle_func_name = format!(
                "{}{}_{}",
                builtin::KCL_SYSTEM_MODULE_MANGLE_PREFIX,
                pkgpath_without_prefix!(pkgpath),
                name
            );
            let value = if pkgpath == builtin::system_module::UNITS
                && builtin::system_module::UNITS_FIELD_NAMES.contains(&name)
            {
                let value_float: f64 = kclvm_runtime::f64_unit_value(name);
                let value_int: u64 = kclvm_runtime::u64_unit_value(name);
                if value_int != 1 {
                    self.int_value(value_int as i64)
                } else {
                    self.float_value(value_float)
                }
            } else {
                let function = self.lookup_function(&mangle_func_name);
                // Convert the function to a i64 pointer to store it into the function value.
                let lambda_fn_ptr = self.builder.build_bitcast(
                    function.as_global_value().as_pointer_value(),
                    self.context.i64_type().ptr_type(AddressSpace::default()),
                    "",
                );
                let func_name = function.get_name().to_str().unwrap();
                let func_name_ptr = self.native_global_string(func_name, func_name).into();
                let none_value = self.none_value();
                self.build_call(
                    &ApiFunc::kclvm_value_Function.name(),
                    &[
                        self.current_runtime_ctx_ptr(),
                        lambda_fn_ptr,
                        none_value,
                        func_name_ptr,
                        self.native_i8_zero().into(),
                    ],
                )
            };
            Ok(value)
        }
        // Plugin pkgpath
        else if pkgpath.starts_with(plugin::PLUGIN_PREFIX_WITH_AT) {
            let null_fn_ptr = self
                .context
                .i64_type()
                .ptr_type(AddressSpace::default())
                .const_zero()
                .into();
            let name = format!("{}.{}", &pkgpath[1..], name);
            let name = self.native_global_string(&name, "").into();
            let none_value = self.none_value();
            return Ok(self.build_call(
                &ApiFunc::kclvm_value_Function.name(),
                &[
                    self.current_runtime_ctx_ptr(),
                    null_fn_ptr,
                    none_value,
                    name,
                    self.native_i8(1).into(),
                ],
            ));
        // User pkgpath
        } else {
            // Global or local variables.
            let scopes = pkg_scopes
                .get(&pkgpath)
                .unwrap_or_else(|| panic!("package {} is not found", pkgpath));
            // Scopes 0 is builtin scope, Scopes 1 is the global scope, Scopes 2~ are the local scopes
            let scopes_len = scopes.len();
            for i in 0..scopes_len {
                let index = scopes_len - i - 1;
                let variables = scopes[index].variables.borrow();
                if let Some(var) = variables.get(&name.to_string()) {
                    // Closure vars, 2 denotes the builtin scope and the global scope, here is a closure scope.
                    let value = if i >= 1 && i < scopes_len - 2 {
                        let last_lambda_scope = self.last_lambda_scope();
                        // Local scope variable
                        if index >= last_lambda_scope {
                            self.builder.build_load(*var, name)
                        } else {
                            // Outer lambda closure
                            let variables = scopes[last_lambda_scope].variables.borrow();
                            let ptr = variables.get(value::LAMBDA_CLOSURE);
                            // Lambda closure
                            match ptr {
                                Some(ptr) => {
                                    let closure_map = self.builder.build_load(*ptr, "");
                                    let string_ptr_value =
                                        self.native_global_string(name, "").into();
                                    // Not a closure, mapbe a local variale
                                    self.build_call(
                                        &ApiFunc::kclvm_dict_get_value.name(),
                                        &[
                                            self.current_runtime_ctx_ptr(),
                                            closure_map,
                                            string_ptr_value,
                                        ],
                                    )
                                }
                                None => self.builder.build_load(*var, name),
                            }
                        }
                    } else {
                        self.builder.build_load(*var, name)
                    };
                    result = Ok(value);
                    break;
                }
            }
            match result {
                Ok(_) => result,
                Err(ref err) => {
                    if !is_in_schema {
                        let mut handler = self.handler.borrow_mut();
                        let pos = Position {
                            filename: self.current_filename(),
                            line: *self.current_line.borrow(),
                            column: None,
                        };
                        handler.add_compile_error(&err.message, (pos.clone(), pos));
                        handler.abort_if_any_errors()
                    }
                    result
                }
            }
        }
    }

    /// Get the variable value named `name` from the scope named `pkgpath`, return Err when not found
    pub fn get_external_variable_in_pkgpath(
        &self,
        name: &str,
        pkgpath: &str,
    ) -> CompileResult<'ctx> {
        let ext_pkgpath = if !pkgpath.starts_with(kclvm_runtime::PKG_PATH_PREFIX)
            && pkgpath != kclvm_runtime::MAIN_PKG_PATH
        {
            format!("{}{}", kclvm_runtime::PKG_PATH_PREFIX, pkgpath)
        } else {
            pkgpath.to_string()
        };
        // System module or plugin module
        if builtin::STANDARD_SYSTEM_MODULE_NAMES_WITH_AT.contains(&ext_pkgpath.as_str())
            || ext_pkgpath.starts_with(plugin::PLUGIN_PREFIX_WITH_AT)
        {
            return self.get_variable_in_pkgpath(name, pkgpath);
        }
        // User module external variable
        let external_var_name = format!("${}.${}", pkgpath_without_prefix!(ext_pkgpath), name);
        let current_pkgpath = self.current_pkgpath();
        let modules = self.modules.borrow();
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let module = &modules.get(&current_pkgpath).expect(&msg).borrow().inner;
        let tpe = self.value_ptr_type();
        let mut global_var_maps = self.global_vars.borrow_mut();
        let pkgpath = self.current_pkgpath();
        if !global_var_maps.contains_key(&pkgpath) {
            global_var_maps.insert(pkgpath.clone(), IndexMap::default());
        }
        // Add or update a external variable
        let global_vars = global_var_maps.get_mut(&pkgpath).expect(&msg);
        let ptr = if let Some(ptr) = global_vars.get(external_var_name.as_str()) {
            *ptr
        } else {
            let global_var =
                module.add_global(tpe, Some(AddressSpace::default()), &external_var_name);
            global_var.set_alignment(GLOBAL_VAL_ALIGNMENT);
            global_var.set_linkage(Linkage::External);
            let ptr = global_var.as_pointer_value();
            global_vars.insert(external_var_name, ptr);
            ptr
        };
        let value = self.builder.build_load(ptr, "");
        Ok(value)
    }

    /// Get closure map in the current inner scope.
    pub(crate) fn get_current_inner_scope_variable_map(&self) -> BasicValueEnum<'ctx> {
        let var_map = {
            let last_lambda_scope = self.last_lambda_scope();
            // Get variable map in the current scope.
            let pkgpath = self.current_pkgpath();
            let pkgpath = if !pkgpath.starts_with(PKG_PATH_PREFIX) && pkgpath != MAIN_PKG_PATH {
                format!("{}{}", PKG_PATH_PREFIX, pkgpath)
            } else {
                pkgpath
            };
            let pkg_scopes = self.pkg_scopes.borrow();
            let scopes = pkg_scopes
                .get(&pkgpath)
                .unwrap_or_else(|| panic!("package {} is not found", pkgpath));
            let current_scope = scopes.len() - 1;
            // Get last closure map.
            let var_map = if current_scope >= last_lambda_scope && last_lambda_scope > 0 {
                let variables = scopes[last_lambda_scope].variables.borrow();
                let ptr = variables.get(value::LAMBDA_CLOSURE);
                let var_map = match ptr {
                    Some(ptr) => self.builder.build_load(*ptr, ""),
                    None => self.dict_value(),
                };
                // Get variable map including schema  in the current scope.
                for i in last_lambda_scope..current_scope + 1 {
                    let variables = scopes
                        .get(i)
                        .expect(kcl_error::INTERNAL_ERROR_MSG)
                        .variables
                        .borrow();
                    for (key, ptr) in &*variables {
                        if key != value::LAMBDA_CLOSURE {
                            let value = self.builder.build_load(*ptr, "");
                            self.dict_insert_override_item(var_map, key.as_str(), value);
                        }
                    }
                }
                var_map
            } else {
                self.dict_value()
            };
            var_map
        };
        // Capture schema `self` closure.
        if self.is_in_schema() {
            for schema_closure_name in value::SCHEMA_VARIABLE_LIST {
                let value = self
                    .get_variable(schema_closure_name)
                    .expect(kcl_error::INTERNAL_ERROR_MSG);
                self.dict_insert_override_item(var_map, schema_closure_name, value);
            }
        }
        var_map
    }

    /// Load value from name.
    pub fn load_value(&self, pkgpath: &str, names: &[&str]) -> CompileResult<'ctx> {
        if names.is_empty() {
            return Err(kcl_error::KCLError {
                message: "error: read value from empty name".to_string(),
                ty: kcl_error::KCLErrorType::Compile,
            });
        }
        let name = names[0];
        // Get variable from the scope.
        let get = |name: &str| {
            match (
                self.is_in_schema(),
                self.is_in_lambda(),
                self.is_local_var(name),
            ) {
                // Get from local or global scope
                (false, _, _) | (_, _, true) => self.get_variable(name),
                // Get variable from the current schema scope.
                (true, false, false) => self.get_variable_in_schema(name),
                // Get from local scope including lambda arguments, lambda variables,
                // loop variables or global variables.
                (true, true, _) =>
                // Get from local scope including lambda arguments, lambda variables,
                // loop variables or global variables.
                {
                    match self.resolve_variable_level(name) {
                        // Closure variable or local variables
                        Some(level) if level > GLOBAL_LEVEL => self.get_variable(name),
                        // Schema closure or global variables
                        _ => self.get_variable_in_schema(name),
                    }
                }
            }
        };
        if names.len() == 1 {
            get(name)
        } else {
            let mut value = if pkgpath.is_empty() {
                get(name)
            } else {
                self.ok_result()
            }
            .expect(kcl_error::INTERNAL_ERROR_MSG);
            for i in 0..names.len() - 1 {
                let attr = names[i + 1];
                if i == 0 && !pkgpath.is_empty() {
                    value = if self.no_link {
                        self.get_external_variable_in_pkgpath(attr, pkgpath)
                    } else {
                        self.get_variable_in_pkgpath(attr, pkgpath)
                    }
                    .expect(kcl_error::INTERNAL_ERROR_MSG)
                } else {
                    let attr = self.native_global_string(attr, "").into();
                    value = self.build_call(
                        &ApiFunc::kclvm_value_load_attr.name(),
                        &[self.current_runtime_ctx_ptr(), value, attr],
                    );
                }
            }
            Ok(value)
        }
    }

    /// Push a lambda definition scope into the lambda stack
    #[inline]
    pub fn push_lambda(&self, scope: usize) {
        self.lambda_stack.borrow_mut().push(scope);
    }

    /// Pop a lambda definition scope.
    #[inline]
    pub fn pop_lambda(&self) {
        self.lambda_stack.borrow_mut().pop();
    }

    #[inline]
    pub fn is_in_lambda(&self) -> bool {
        *self
            .lambda_stack
            .borrow()
            .last()
            .expect(kcl_error::INTERNAL_ERROR_MSG)
            > GLOBAL_LEVEL
    }

    #[inline]
    pub fn last_lambda_scope(&self) -> usize {
        *self
            .lambda_stack
            .borrow()
            .last()
            .expect(kcl_error::INTERNAL_ERROR_MSG)
    }

    #[inline]
    pub fn is_in_schema(&self) -> bool {
        self.schema_stack.borrow().len() > 0
    }

    #[inline]
    pub fn is_in_schema_expr(&self) -> bool {
        self.schema_expr_stack.borrow().len() > 0
    }

    #[inline]
    pub fn is_local_var(&self, name: &str) -> bool {
        self.local_vars.borrow().contains(name)
    }

    /// Push a function call frame into the function stack
    #[inline]
    pub fn push_function(&self, function: FunctionValue<'ctx>) {
        self.functions.borrow_mut().push(Rc::new(function));
    }

    /// Pop a function from the function stack
    #[inline]
    pub fn pop_function(&self) {
        self.functions.borrow_mut().pop();
    }

    /// Get the current function
    #[inline]
    pub fn current_function(&self) -> FunctionValue<'ctx> {
        **self
            .functions
            .borrow()
            .last()
            .expect(kcl_error::INTERNAL_ERROR_MSG)
    }

    /// Plan globals to a json string
    pub fn globals_to_json_str(&self) -> BasicValueEnum<'ctx> {
        let current_pkgpath = self.current_pkgpath();
        let pkg_scopes = self.pkg_scopes.borrow();
        let scopes = pkg_scopes
            .get(&current_pkgpath)
            .unwrap_or_else(|| panic!("pkgpath {} is not found", current_pkgpath));
        // The global scope.
        let scope = scopes.last().expect(kcl_error::INTERNAL_ERROR_MSG);
        let scalars = scope.scalars.borrow();
        let globals = scope.variables.borrow();
        // Construct a plan object.
        let global_dict = self.dict_value();
        // Plan empty dict result.
        if scalars.is_empty() && globals.is_empty() {
            return self.build_call(
                &ApiFunc::kclvm_value_plan_to_json.name(),
                &[self.current_runtime_ctx_ptr(), global_dict],
            );
        }
        // Deal scalars
        for scalar in scalars.iter() {
            self.dict_safe_insert(global_dict, SCALAR_KEY, *scalar, 0, -1);
        }
        // Deal global variables
        for (name, ptr) in globals.iter() {
            let value = self.builder.build_load(*ptr, "");
            let value_dict = self.dict_value();
            self.dict_safe_insert(value_dict, name.as_str(), value, 0, -1);
            self.dict_safe_insert(global_dict, SCALAR_KEY, value_dict, 0, -1);
        }
        // Plan result to json string.
        self.build_call(
            &ApiFunc::kclvm_value_plan_to_json.name(),
            &[
                self.current_runtime_ctx_ptr(),
                self.dict_get(
                    global_dict,
                    self.native_global_string(SCALAR_KEY, "").into(),
                ),
            ],
        )
    }

    /// Insert a dict entry including key, value, op and insert_index into the dict.
    #[inline]
    fn dict_safe_insert(
        &self,
        dict: BasicValueEnum<'ctx>,
        key: &str,
        value: BasicValueEnum<'ctx>,
        op: i32,
        insert_index: i32,
    ) {
        let name = self.native_global_string(key, "").into();
        let op = self.native_int_value(op);
        let insert_index = self.native_int_value(insert_index);
        self.build_void_call(
            &ApiFunc::kclvm_dict_safe_insert.name(),
            &[
                self.current_runtime_ctx_ptr(),
                dict,
                name,
                value,
                op,
                insert_index,
            ],
        );
    }

    /// Merge a dict entry including key, value, op and insert_index into the dict
    /// without the idempotent check.
    #[inline]
    pub fn dict_merge(
        &self,
        dict: BasicValueEnum<'ctx>,
        key: &str,
        value: BasicValueEnum<'ctx>,
        op: i32,
        insert_index: i32,
    ) {
        let name = self.native_global_string(key, "").into();
        let op = self.native_int_value(op);
        let insert_index = self.native_int_value(insert_index);
        self.build_void_call(
            &ApiFunc::kclvm_dict_merge.name(),
            &[
                self.current_runtime_ctx_ptr(),
                dict,
                name,
                value,
                op,
                insert_index,
            ],
        );
    }

    /// default_dict(list) insert a key-value pair, and the value is a int pointer
    #[inline]
    pub fn default_collection_insert_int_pointer(
        &self,
        dict: BasicValueEnum<'ctx>,
        key: &str,
        value: BasicValueEnum<'ctx>,
    ) {
        let name = self.native_global_string(key, "").into();
        self.build_void_call(
            ApiFunc::kclvm_default_collection_insert_int_pointer
                .name()
                .as_str(),
            &[dict, name, value],
        );
    }

    /// default_dict(list) insert a key-value pair
    #[inline]
    pub fn default_collection_insert_value(
        &self,
        dict: BasicValueEnum<'ctx>,
        key: &str,
        value: BasicValueEnum<'ctx>,
    ) {
        let name = self.native_global_string(key, "").into();
        self.build_void_call(
            ApiFunc::kclvm_default_collection_insert_value
                .name()
                .as_str(),
            &[dict, name, value],
        );
    }
}
