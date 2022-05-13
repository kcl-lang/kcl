//! Copyright 2021 The KCL Authors. All rights reserved.

use crate::codegen::abi::AddressSpace;
use crate::codegen::{CONTEXT_TYPE_NAME, VALUE_TYPE_NAME};

use super::BackendTypes;

/// BaseTypeMethods defines all native base type APIs, e.g. i8/i16/f32/f64 types, etc.
pub trait BaseTypeMethods: BackendTypes {
    /// Native i8 type
    fn i8_type(&self) -> Self::Type;
    /// Native i16 type
    fn i16_type(&self) -> Self::Type;
    /// Native i32 type
    fn i32_type(&self) -> Self::Type;
    /// Native i64 type
    fn i64_type(&self) -> Self::Type;
    /// Native i128 type
    fn i128_type(&self) -> Self::Type;
    /// Native f32 type
    fn f32_type(&self) -> Self::Type;
    /// Native f64 type
    fn f64_type(&self) -> Self::Type;
    /// Native struct type.
    fn struct_type(&self, els: &[Self::Type], packed: bool) -> Self::Type;
    /// Native pointer type of `ty`.
    fn ptr_type_to(&self, ty: Self::Type) -> Self::Type;
    /// Native pointer type of `ty` with the address space.
    fn ptr_type_to_ext(&self, ty: Self::Type, address_space: AddressSpace) -> Self::Type;
    /// Native array element type.
    fn element_type(&self, ty: Self::Type) -> Self::Type;
    /// Returns the number of elements in `self`.
    fn vector_length(&self, ty: Self::Type) -> usize;
    /// Retrieves the bit width of the float type `self`.
    fn float_width(&self, ty: Self::Type) -> usize;
    /// Retrieves the bit width of the integer type `self`.
    fn int_width(&self, ty: Self::Type) -> usize;
    /// Get the value type.
    fn val_type(&self, v: Self::Value) -> Self::Type;
    /// Native function type
    fn function_let(&self, args: &[Self::Type], ret: Self::Type) -> Self::FunctionLet;
}

/// DerivedTypeMethods defines all extended type APIs.
pub trait DerivedTypeMethods: BaseTypeMethods {
    /// Lookup a intrinsic type by the name.
    fn get_intrinsic_type(&self, name: &str) -> Self::Type;
    /// Get the value pointer type.
    fn value_ptr_type(&self) -> Self::Type {
        self.ptr_type_to(self.get_intrinsic_type(VALUE_TYPE_NAME))
    }
    /// Get the context pointer type.
    fn context_ptr_type(&self) -> Self::Type {
        self.ptr_type_to(self.get_intrinsic_type(CONTEXT_TYPE_NAME))
    }
    /// Get the function type.
    fn function_type(&self) -> Self::FunctionLet {
        let value_ptr_type = self.value_ptr_type();
        let context_ptr_type = self.context_ptr_type();
        self.function_let(
            &[context_ptr_type, value_ptr_type, value_ptr_type],
            value_ptr_type,
        )
    }
}

/// TypeCodeGen defines all type APIs.
pub trait TypeCodeGen: DerivedTypeMethods {}
