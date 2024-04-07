use std::sync::Arc;

use crate::resolver::Resolver;
use crate::ty::{
    has_any_type, is_upper_bound, sup, Type, TypeInferMethods, TypeRef, ZERO_LIT_TYPES,
};
use kclvm_ast::ast;
use kclvm_error::diagnostic::Range;

const DIV_OR_MOD_ZERO_MSG: &str = "integer division or modulo by zero";

impl<'ctx> Resolver<'ctx> {
    /// Binary operator calculation table.
    ///
    /// Arithmetic (int or float; result has type float unless both operands have type int)
    ///    number + number              # addition
    ///    number - number              # subtraction
    ///    number * number              # multiplication
    ///    number / number              # real division  (result is always a float)
    ///    number // number             # floored division
    ///    number % number              # remainder of floored division
    ///    number ^ number              # bitwise XOR
    ///    number << number             # bitwise left shift
    ///    number >> number             # bitwise right shift
    ///
    /// Concatenation
    ///     string + string
    ///     list + list
    ///
    /// Repetition (string/list)
    ///     int * sequence
    ///     sequence * int
    ///
    /// Union
    ///     int | int
    ///     list | list
    ///     dict | dict
    ///     schema | schema
    ///     schema | dict
    ///
    /// Add: number + number, str + str, list + list
    /// Sub: number - number
    /// Mul: number * number, int * list, list * int, int * str, str * int
    /// Div: number / number
    /// FloorDiv: number // number
    /// Mod: number % number
    /// Pow: number ** number
    /// LShift: int >> int
    /// RShift: int << int
    /// BitOr: int | int, list | list, dict | dict, schema | schema, schema | dict
    /// BitXOr: int ^ int
    /// BitAdd int & int
    ///
    /// And: any_type and any_type -> bool
    /// Or: any_type1 or any_type1 -> sup([any_type1, any_type2])
    pub fn binary(
        &mut self,
        left: TypeRef,
        right: TypeRef,
        op: &ast::BinOp,
        range: Range,
    ) -> TypeRef {
        let t1 = self
            .ctx
            .ty_ctx
            .literal_union_type_to_variable_type(left.clone());
        let t2 = self
            .ctx
            .ty_ctx
            .literal_union_type_to_variable_type(right.clone());
        if has_any_type(&[t1.clone(), t2.clone()]) {
            return self.any_ty();
        }
        let number_binary = |left: &TypeRef, right: &TypeRef| {
            if left.is_float() || right.is_float() {
                Arc::new(Type::FLOAT)
            } else {
                Arc::new(Type::INT)
            }
        };
        let (result, return_ty) = match op {
            ast::BinOp::Add => {
                if t1.is_number() && t2.is_number() {
                    (true, number_binary(&t1, &t2))
                } else if t1.is_str() && t2.is_str() {
                    (true, self.str_ty())
                } else if t1.is_list() && t2.is_list() {
                    (
                        true,
                        Type::list_ref(sup(&[t1.list_item_ty(), t2.list_item_ty()])),
                    )
                } else {
                    (false, self.any_ty())
                }
            }
            ast::BinOp::Sub | ast::BinOp::Pow => {
                if t1.is_number() && t2.is_number() {
                    (true, number_binary(&t1, &t2))
                } else {
                    (false, self.any_ty())
                }
            }
            ast::BinOp::Mul => {
                if t1.is_number() && t2.is_number() {
                    (true, number_binary(&t1, &t2))
                } else if t1.is_int()
                    && self
                        .ctx
                        .ty_ctx
                        .is_mul_val_type_or_mul_val_union_type(t2.clone())
                {
                    (true, t2)
                } else if self
                    .ctx
                    .ty_ctx
                    .is_mul_val_type_or_mul_val_union_type(t1.clone())
                    && t2.is_int()
                {
                    (true, t1)
                } else {
                    (false, self.any_ty())
                }
            }
            ast::BinOp::Div | ast::BinOp::FloorDiv => {
                if t1.is_number() && t2.is_number() {
                    if ZERO_LIT_TYPES.contains(&t2) {
                        self.handler
                            .add_type_error(DIV_OR_MOD_ZERO_MSG, range.clone());
                    }
                    (true, number_binary(&t1, &t2))
                } else {
                    (false, self.any_ty())
                }
            }
            ast::BinOp::Mod => {
                if t1.is_number() && t2.is_number() {
                    if ZERO_LIT_TYPES.contains(&t2) {
                        self.handler
                            .add_type_error(DIV_OR_MOD_ZERO_MSG, range.clone());
                    }
                    (true, self.int_ty())
                } else {
                    (false, self.any_ty())
                }
            }
            ast::BinOp::LShift | ast::BinOp::RShift | ast::BinOp::BitXor | ast::BinOp::BitAnd => {
                if t1.is_int() && t2.is_int() {
                    (true, self.int_ty())
                } else {
                    (false, self.any_ty())
                }
            }
            ast::BinOp::BitOr => {
                if t1.is_int() && t2.is_int() {
                    (true, self.int_ty())
                } else if t1.is_none() {
                    (true, t2)
                } else if t2.is_none() {
                    (true, t1)
                } else if t1.is_list() && t2.is_list() {
                    (
                        true,
                        Type::list_ref(sup(&[t1.list_item_ty(), t2.list_item_ty()])),
                    )
                } else if t1.is_dict() && t2.is_dict() {
                    let (t1_key_ty, t1_val_ty) = t1.dict_entry_ty();
                    let (t2_key_ty, t2_val_ty) = t2.dict_entry_ty();
                    (
                        true,
                        Type::dict_ref(sup(&[t1_key_ty, t2_key_ty]), sup(&[t1_val_ty, t2_val_ty])),
                    )
                } else if t1.is_schema() && (t2.is_schema() || t2.is_dict()) {
                    (true, t1)
                } else {
                    (false, self.any_ty())
                }
            }
            ast::BinOp::And => (true, self.bool_ty()),
            ast::BinOp::Or => (true, sup(&[t1, t2])),
            ast::BinOp::As => {
                if !is_upper_bound(
                    self.ctx.ty_ctx.infer_to_variable_type(t1.clone()),
                    t2.clone(),
                ) {
                    self.handler.add_type_error(
                        &format!(
                            "Conversion of type '{}' to type '{}' may be a mistake because neither type sufficiently overlaps with the other",
                            t1.ty_str(),
                            t2.ty_str()
                        ),
                        range.clone(),
                    );
                }
                (true, t2)
            }
        };

        if !result {
            self.handler.add_type_error(
                &format!(
                    "unsupported operand type(s) for {}: '{}' and '{}'",
                    op.symbol(),
                    left.ty_str(),
                    right.ty_str()
                ),
                range,
            );
        }
        return_ty
    }

    /// Unary operator calculation table
    ///
    /// + number        unary positive          (int, float)
    /// - number        unary negation          (int, float)
    /// ~ number        unary bitwise inversion (int)
    /// not x           logical negation        (any type)
    pub fn unary(&mut self, ty: TypeRef, op: &ast::UnaryOp, range: Range) -> TypeRef {
        if has_any_type(&[ty.clone()]) {
            return self.any_ty();
        }
        let var_ty = self
            .ctx
            .ty_ctx
            .literal_union_type_to_variable_type(ty.clone());
        let result = match op {
            ast::UnaryOp::UAdd | ast::UnaryOp::USub => var_ty.is_number(),
            ast::UnaryOp::Invert => var_ty.is_int() || var_ty.is_bool(),
            ast::UnaryOp::Not => true,
        };
        if result {
            var_ty
        } else {
            self.handler.add_type_error(
                &format!(
                    "bad operand type for unary {}: '{}'",
                    op.symbol(),
                    ty.ty_str(),
                ),
                range,
            );
            self.any_ty()
        }
    }

    /// Compare operator calculation table
    ///
    /// int                 # mathematical            1 < 2
    /// float               # as defined by IEEE 754  1.0 < 2.0
    /// list/config/schema  # lexicographical         [1] == [2]
    /// iterable            # 1 in [1, 2, 3], "s" in "ss", "key" in Schema
    /// relation            # a is True, b is Undefined
    pub fn compare(
        &mut self,
        left: TypeRef,
        right: TypeRef,
        op: &ast::CmpOp,
        range: Range,
    ) -> TypeRef {
        let t1 = self.ctx.ty_ctx.literal_union_type_to_variable_type(left);
        let t2 = self.ctx.ty_ctx.literal_union_type_to_variable_type(right);
        if has_any_type(&[t1.clone(), t2.clone()]) {
            return self.any_ty();
        }
        if self
            .ctx
            .ty_ctx
            .is_number_bool_type_or_number_bool_union_type(t1.clone())
            && self
                .ctx
                .ty_ctx
                .is_number_bool_type_or_number_bool_union_type(t2.clone())
            && !matches!(op, ast::CmpOp::In | ast::CmpOp::NotIn)
        {
            return self.bool_ty();
        }
        if self
            .ctx
            .ty_ctx
            .is_primitive_type_or_primitive_union_type(t1.clone())
            && self
                .ctx
                .ty_ctx
                .is_primitive_type_or_primitive_union_type(t2.clone())
            && matches!(op, ast::CmpOp::Eq | ast::CmpOp::NotEq)
        {
            return self.bool_ty();
        }
        if matches!(op, ast::CmpOp::Eq) && t1.is_list() && t2.is_list() {
            return self.bool_ty();
        }
        if matches!(op, ast::CmpOp::Eq) && t1.is_dict_or_schema() && t2.is_dict_or_schema() {
            return self.bool_ty();
        }
        if matches!(op, ast::CmpOp::In | ast::CmpOp::NotIn) && t2.is_iterable() {
            return self.bool_ty();
        }
        if (t1.is_none() || t2.is_none())
            && matches!(
                op,
                ast::CmpOp::Eq
                    | ast::CmpOp::NotEq
                    | ast::CmpOp::Is
                    | ast::CmpOp::IsNot
                    | ast::CmpOp::Not
            )
        {
            return self.bool_ty();
        }
        self.handler.add_type_error(
            &format!(
                "unsupported operand type(s) for {}: '{}' and '{}'",
                op.symbol(),
                t1.ty_str(),
                t2.ty_str(),
            ),
            range,
        );
        self.any_ty()
    }
}
