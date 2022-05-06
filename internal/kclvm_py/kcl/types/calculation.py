# Copyright 2020 The KCL Authors. All rights reserved.

from typing import Union, List, Optional

import kclvm.kcl.error as kcl_error
import kclvm.kcl.ast as ast
import kclvm.api.object as objpkg

from .type import (
    Type,
    ANY_TYPE,
    INT_TYPE,
    FLOAT_TYPE,
    STR_TYPE,
    BOOL_TYPE,
    FALSE_LIT_TYPE,
    NONE_TYPE,
    NUMBER_TYPE_KINDS,
    ITERABLE_KINDS,
    STR_KINDS,
    INT_KINDS,
    BOOL_KINDS,
    BUILTIN_KINDS,
    sup,
    has_any_type,
    is_upper_bound,
    infer_to_variable_type,
    literal_union_type_to_variable_type,
    is_kind_type_or_kind_union_type,
)


ZERO_LIT_TYPES: List[Type] = [
    objpkg.KCLIntLitTypeObject(0),
    objpkg.KCLFloatLitTypeObject(0.0),
    FALSE_LIT_TYPE,
]


def binary(
    t1: Type,
    t2: Type,
    op: Union[ast.BinOp, ast.AugOp],
    filename: Optional[str] = None,
    line: Optional[int] = None,
    column: Optional[int] = None,
) -> Type:
    """Binary operator calculation table.

    Arithmetic (int or float; result has type float unless both operands have type int)
       number + number              # addition
       number - number              # subtraction
       number * number              # multiplication
       number / number              # real division  (result is always a float)
       number // number             # floored division
       number % number              # remainder of floored division
       number ^ number              # bitwise XOR
       number << number             # bitwise left shift
       number >> number             # bitwise right shift

    Concatenation
        string + string
        list + list

    Repetition (string/list)
        int * sequence
        sequence * int

    Union
        int | int
        list | list
        dict | dict
        schema | schema
        schema | dict

    Add: number + number, str + str, list + list
    Sub: number - number
    Mul: number * number, number * list, list * number
    Div: number / number
    FloorDiv: number // number
    Mod: number % number
    Pow: number ** number
    LShift: int >> int
    RShift: int << int
    BitOr: int | int, list | list, dict | dict, schema | schema, schema | dict
    BitXOr: int ^ int
    BitAdd int & int

    And: any_type and any_type -> bool
    Or: any_type1 or any_type1 -> sup([any_type1, any_type2])
    """

    def number_binary() -> Type:
        return (
            FLOAT_TYPE
            if (
                t1.type_kind()
                in [objpkg.KCLTypeKind.FloatKind, objpkg.KCLTypeKind.FloatLitKind]
                or t2.type_kind()
                in [objpkg.KCLTypeKind.FloatKind, objpkg.KCLTypeKind.FloatLitKind]
            )
            else INT_TYPE
        )

    if has_any_type([t1, t2]):
        return ANY_TYPE

    t1 = literal_union_type_to_variable_type(t1)
    t2 = literal_union_type_to_variable_type(t2)

    if op == ast.BinOp.Add or op == ast.AugOp.Add:
        if t1.type_kind() in NUMBER_TYPE_KINDS and t2.type_kind() in NUMBER_TYPE_KINDS:
            return number_binary()
        if t1.type_kind() in STR_KINDS and t2.type_kind() in STR_KINDS:
            return STR_TYPE
        if isinstance(t1, objpkg.KCLListTypeObject) and isinstance(
            t2, objpkg.KCLListTypeObject
        ):
            return objpkg.KCLListTypeObject(item_type=sup([t1.item_type, t2.item_type]))
    elif op == ast.BinOp.Mul or op == ast.AugOp.Mul:
        if t1.type_kind() in NUMBER_TYPE_KINDS and t2.type_kind() in NUMBER_TYPE_KINDS:
            return number_binary()
        if t1.type_kind() in INT_KINDS and (
            is_kind_type_or_kind_union_type(
                t2, STR_KINDS + NUMBER_TYPE_KINDS + [objpkg.KCLTypeKind.ListKind]
            )
        ):
            return t2
        if (
            is_kind_type_or_kind_union_type(
                t1, STR_KINDS + NUMBER_TYPE_KINDS + [objpkg.KCLTypeKind.ListKind]
            )
        ) and t2.type_kind() in INT_KINDS:
            return t1
    elif op == ast.BinOp.Sub or op == ast.AugOp.Sub:
        if t1.type_kind() in NUMBER_TYPE_KINDS and t2.type_kind() in NUMBER_TYPE_KINDS:
            return number_binary()
    elif op == ast.BinOp.Div or op == ast.AugOp.Div:
        if t1.type_kind() in NUMBER_TYPE_KINDS and t2.type_kind() in NUMBER_TYPE_KINDS:
            if t2 in ZERO_LIT_TYPES:
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.CompileError_TYPE,
                    file_msgs=[
                        kcl_error.ErrFileMsg(
                            filename=filename, line_no=line, col_no=column
                        )
                    ],
                    arg_msg="integer division or modulo by zero",
                )
            return number_binary()
    elif op == ast.BinOp.Mod or op == ast.AugOp.Mod:
        if t1.type_kind() in NUMBER_TYPE_KINDS and t2.type_kind() in NUMBER_TYPE_KINDS:
            if t2 in ZERO_LIT_TYPES:
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.CompileError_TYPE,
                    file_msgs=[
                        kcl_error.ErrFileMsg(
                            filename=filename, line_no=line, col_no=column
                        )
                    ],
                    arg_msg="integer division or modulo by zero",
                )
            return INT_TYPE
    elif op == ast.BinOp.FloorDiv or op == ast.AugOp.FloorDiv:
        if t1.type_kind() in NUMBER_TYPE_KINDS and t2.type_kind() in NUMBER_TYPE_KINDS:
            if t2 in ZERO_LIT_TYPES:
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.CompileError_TYPE,
                    file_msgs=[
                        kcl_error.ErrFileMsg(
                            filename=filename, line_no=line, col_no=column
                        )
                    ],
                    arg_msg="integer division or modulo by zero",
                )
            return number_binary()
    elif op == ast.BinOp.Pow or op == ast.AugOp.Pow:
        if t1.type_kind() in NUMBER_TYPE_KINDS and t2.type_kind() in NUMBER_TYPE_KINDS:
            return number_binary()
    elif op == ast.BinOp.BitOr or op == ast.AugOp.BitOr:
        if t1.type_kind() in INT_KINDS and t2.type_kind() in INT_KINDS:
            return INT_TYPE
        if t2 == NONE_TYPE:
            return t1
        if t1 == NONE_TYPE:
            return t2
        if isinstance(t1, objpkg.KCLListTypeObject) and isinstance(
            t2, objpkg.KCLListTypeObject
        ):
            return objpkg.KCLListTypeObject(item_type=sup([t1.item_type, t2.item_type]))
        if isinstance(t1, objpkg.KCLDictTypeObject) and isinstance(
            t2, objpkg.KCLDictTypeObject
        ):
            return objpkg.KCLDictTypeObject(
                key_type=sup([t1.key_type, t2.key_type]),
                value_type=sup([t1.value_type, t2.value_type]),
            )
        if isinstance(t1, objpkg.KCLSchemaTypeObject) and isinstance(
            t2, (objpkg.KCLSchemaTypeObject, objpkg.KCLDictTypeObject)
        ):
            return t1
    elif op == ast.BinOp.LShift or op == ast.AugOp.LShift:
        if t1.type_kind() in INT_KINDS and t2.type_kind() in INT_KINDS:
            return INT_TYPE
    elif op == ast.BinOp.RShift or op == ast.AugOp.RShift:
        if t1.type_kind() in INT_KINDS and t2.type_kind() in INT_KINDS:
            return INT_TYPE
    elif op == ast.BinOp.BitXor or op == ast.AugOp.BitXor:
        if t1.type_kind() in INT_KINDS and t2.type_kind() in INT_KINDS:
            return INT_TYPE
    elif op == ast.BinOp.BitAnd or op == ast.AugOp.BitAnd:
        if t1.type_kind() in INT_KINDS and t2.type_kind() in INT_KINDS:
            return INT_TYPE
    elif op == ast.BinOp.And:
        return BOOL_TYPE
    elif op == ast.BinOp.Or:
        return sup([t1, t2])
    elif op == ast.BinOp.As:
        if not is_upper_bound(infer_to_variable_type(t1), t2):
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.CompileError_TYPE,
                file_msgs=[
                    kcl_error.ErrFileMsg(filename=filename, line_no=line, col_no=column)
                ],
                arg_msg=f"Conversion of type '{t1.type_str()}' to type '{t2.type_str()}' "
                "may be a mistake because neither type sufficiently overlaps with the other",
            )
        return t2.schema_type if isinstance(t2, objpkg.KCLSchemaDefTypeObject) else t2
    kcl_error.report_exception(
        err_type=kcl_error.ErrType.CompileError_TYPE,
        file_msgs=[
            kcl_error.ErrFileMsg(filename=filename, line_no=line, col_no=column)
        ],
        arg_msg=f"unsupported operand type(s) for {ast.OPERATOR_VALUE_MAP.get(op)}: '{t1.type_str()}' and '{t2.type_str()}'",
    )


def compare(
    t1: Type,
    t2: Type,
    op: ast.CmpOp,
    filename: Optional[str] = None,
    line: Optional[int] = None,
    column: Optional[int] = None,
) -> Type:
    """Compare operator calculation table

    bool            # False < True            False < True
    int             # mathematical            1 < 2
    float           # as defined by IEEE 754  1.0 < 2.0
    string          # lexicographical         "1" < 2
    list            # lexicographical         [1] == [2]
    iterable        # 1 in [1, 2, 3], "s" in "ss", "key" in Schema
    """

    t1 = literal_union_type_to_variable_type(t1)
    t2 = literal_union_type_to_variable_type(t2)

    if has_any_type([t1, t2]):
        return ANY_TYPE
    if (
        is_kind_type_or_kind_union_type(t1, NUMBER_TYPE_KINDS)
        and is_kind_type_or_kind_union_type(t2, NUMBER_TYPE_KINDS)
        and op not in [ast.CmpOp.In, ast.CmpOp.NotIn]
    ):
        return BOOL_TYPE
    if (
        is_kind_type_or_kind_union_type(t1, STR_KINDS)
        and is_kind_type_or_kind_union_type(t2, STR_KINDS)
        and op not in [ast.CmpOp.Eq, ast.CmpOp.NotEq]
    ):
        return BOOL_TYPE
    if (
        is_kind_type_or_kind_union_type(t1, BUILTIN_KINDS)
        and is_kind_type_or_kind_union_type(t2, BUILTIN_KINDS)
        and op in [ast.CmpOp.Eq, ast.CmpOp.NotEq]
    ):
        return BOOL_TYPE
    if isinstance(t1, objpkg.KCLListTypeObject) and isinstance(
        t2, objpkg.KCLListTypeObject
    ):
        return BOOL_TYPE
    if (
        isinstance(t1, (objpkg.KCLDictTypeObject, objpkg.KCLSchemaTypeObject))
        and isinstance(t2, (objpkg.KCLDictTypeObject, objpkg.KCLSchemaTypeObject))
        and op in [ast.CmpOp.Eq, ast.CmpOp.NotEq]
    ):
        return BOOL_TYPE
    if op in [ast.CmpOp.In, ast.CmpOp.NotIn] and t2.type_kind() in ITERABLE_KINDS:
        return BOOL_TYPE
    if (t1 == NONE_TYPE or t2 == NONE_TYPE) and op in [
        ast.CmpOp.Eq,
        ast.CmpOp.NotEq,
        ast.CmpOp.Is,
        ast.CmpOp.IsNot,
        ast.CmpOp.Not,
    ]:
        return BOOL_TYPE
    kcl_error.report_exception(
        err_type=kcl_error.ErrType.CompileError_TYPE,
        file_msgs=[
            kcl_error.ErrFileMsg(filename=filename, line_no=line, col_no=column)
        ],
        arg_msg=f"unsupported operand type(s) for {ast.OPERATOR_VALUE_MAP.get(op)}: '{t1.type_str()}' and '{t2.type_str()}'",
    )


def unary(
    t: Type,
    op: ast.UnaryOp,
    filename: Optional[str] = None,
    line: Optional[int] = None,
    column: Optional[int] = None,
) -> Type:
    """Unary operator calculation table

    + number        unary positive          (int, float)
    - number        unary negation          (int, float)
    ~ number        unary bitwise inversion (int)
    not x           logical negation        (any type)
    """
    if has_any_type([t]):
        return ANY_TYPE

    t = literal_union_type_to_variable_type(t)

    if op == ast.UnaryOp.UAdd:
        if t.type_kind() in NUMBER_TYPE_KINDS:
            return t
    if op == ast.UnaryOp.USub:
        if t.type_kind() in NUMBER_TYPE_KINDS:
            return t
    if op == ast.UnaryOp.Invert:
        if t.type_kind() in (INT_KINDS + BOOL_KINDS):
            return INT_TYPE
    if op == ast.UnaryOp.Not:
        return BOOL_TYPE
    kcl_error.report_exception(
        err_type=kcl_error.ErrType.CompileError_TYPE,
        file_msgs=[
            kcl_error.ErrFileMsg(filename=filename, line_no=line, col_no=column)
        ],
        arg_msg=f"bad operand type for unary {ast.OPERATOR_VALUE_MAP.get(op)}: '{t.type_str()}'",
    )
