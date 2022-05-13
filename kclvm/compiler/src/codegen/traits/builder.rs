//! Copyright 2021 The KCL Authors. All rights reserved.

use crate::codegen::abi::Align;

use super::BackendTypes;
/// BuilderMethods defines SSA builder methods including calculation, condition, SSA instructions etc.
pub trait BuilderMethods: BackendTypes {
    /// SSA append a basic block named `name`.
    fn append_block(&self, name: &str) -> Self::BasicBlock;
    /// SSA switch to the block.
    fn switch_to_block(&self, block: Self::BasicBlock);
    /// SSA alloca instruction.
    fn alloca(&self, ty: Self::Type, name: &str, align: Option<Align>) -> Self::Value;
    /// SSA array alloca instruction.
    fn array_alloca(
        &self,
        ty: Self::Type,
        len: Self::Value,
        name: &str,
        align: Align,
    ) -> Self::Value;
    /// SSA ret instruction.
    fn ret_void(&self);
    /// SSA ret instruction with returned value.
    fn ret(&self, v: Self::Value);
    /// SSA br instruction.
    fn br(&self, dest: Self::BasicBlock);
    /// SSA cond br instruction.
    fn cond_br(&self, cond: Self::Value, then_bb: Self::BasicBlock, else_bb: Self::BasicBlock);
    /// SSA select instruction.
    fn select(
        &self,
        cond: Self::Value,
        then_val: Self::Value,
        else_val: Self::Value,
    ) -> Self::Value;
    /// SSA va arg instruction.
    fn va_arg(&self, list: Self::Value, ty: Self::Type) -> Self::Value;
    /// SSA extract element instruction.
    fn extract_element(&self, vec: Self::Value, idx: Self::Value) -> Self::Value;
    /// SSA extract value instruction.
    fn extract_value(&self, agg_val: Self::Value, idx: u32) -> Self::Value;
    /// SSA insert value instruction.
    fn insert_value(&self, agg_val: Self::Value, elt: Self::Value, idx: u32) -> Self::Value;
    /// SSA function invoke instruction.
    fn invoke(
        &self,
        ty: Self::Type,
        fn_value: Self::Function,
        args: &[Self::Value],
        then: Self::BasicBlock,
        catch: Self::BasicBlock,
    ) -> Self::Value;
    /// SSA function call instruction.
    fn call(&self, ty: Self::Type, fn_value: Self::Function, args: &[Self::Value]) -> Self::Value;
    /// SSA load instruction.
    fn load(&self, ptr: Self::Value, name: &str) -> Self::Value;
    /// SSA store instruction.
    fn store(&self, ptr: Self::Value, val: Self::Value);
    /// SSA gep instruction.
    fn gep(&self, ty: Self::Type, ptr: Self::Value, indices: &[Self::Value]) -> Self::Value;
    /// SSA inbounds gep instruction.
    fn inbounds_gep(
        &self,
        ty: Self::Type,
        ptr: Self::Value,
        indices: &[Self::Value],
    ) -> Self::Value;
    /// SSA struct gep instruction.
    fn struct_gep(&self, ty: Self::Type, ptr: Self::Value, idx: u32) -> Self::Value;
    /// SSA cast pointer to int.
    fn ptr_to_int(&self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;
    /// SSA cast int to pointer.
    fn int_to_ptr(&self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;
    /// SSA bit cast.
    fn bit_cast(&self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;
    /// SSA int cast.
    fn int_cast(&self, val: Self::Value, dest_ty: Self::Type, is_signed: bool) -> Self::Value;
    /// SSA pointer cast.
    fn ptr_cast(&self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;
    /// Lookup a known function named `name`.
    fn lookup_function(&self, name: &str) -> Self::Function;
    /// Add a function named `name`.
    fn add_function(&self, name: &str) -> Self::Function;
}
