from enum import IntEnum
from typing import Union

import kclvm.kcl.ast.ast as ast


class OpPrecedence(IntEnum):
    """Operator precedence in grammar

    assign_stmt:  // precedence 0
    or_test: and_test (L_OR and_test)*  // precedence 1
    and_test: not_test (L_AND not_test)*  // precedence 2
    not_test: L_NOT not_test | comparison  // precedence 3
    comparison: expr (comp_op expr)*  // precedence 4
    expr: xor_expr (OR xor_expr)*  // precedence 5
    xor_expr: and_expr (XOR and_expr)*  // precedence 6
    and_expr: shift_expr (AND shift_expr)*  // precedence 7
    shift_expr: arith_expr ((SHIFT_LEFT|SHIFT_RIGHT) arith_expr)*  // precedence 8
    arith_expr: term ((PLUS|MINUS) term)*  // precedence 9
    term: factor ((MULTIPLY|DIVIDE|MOD|DOUBLE_DIVIDE) factor)*  // precedence 10
    factor: (PLUS|MINUS|NOT) factor | power  // precedence 11
    power: primary_expr (DOUBLE_STAR factor)?  // precedence 12
    """

    LOWEST = 0
    HIGHEST = 12


OP_PREC_MAP = {
    ast.AugOp.Assign: 0,
    ast.AugOp.Add: 0,
    ast.AugOp.Sub: 0,
    ast.AugOp.Mul: 0,
    ast.AugOp.Div: 0,
    ast.AugOp.Mod: 0,
    ast.AugOp.Pow: 0,
    ast.AugOp.LShift: 0,
    ast.AugOp.RShift: 0,
    ast.AugOp.BitOr: 0,
    ast.AugOp.BitXor: 0,
    ast.AugOp.BitAnd: 0,
    ast.AugOp.FloorDiv: 0,
    ast.BinOp.Add: 9,
    ast.BinOp.Sub: 9,
    ast.BinOp.Mul: 10,
    ast.BinOp.Div: 10,
    ast.BinOp.Mod: 10,
    ast.BinOp.Pow: 12,
    ast.BinOp.LShift: 8,
    ast.BinOp.RShift: 8,
    ast.BinOp.BitOr: 6,
    ast.BinOp.BitXor: 5,
    ast.BinOp.BitAnd: 7,
    ast.BinOp.FloorDiv: 10,
    ast.BinOp.As: 4,
    ast.BinOp.And: 2,
    ast.BinOp.Or: 1,
    ast.CmpOp.Eq: 4,
    ast.CmpOp.NotEq: 4,
    ast.CmpOp.Lt: 4,
    ast.CmpOp.LtE: 4,
    ast.CmpOp.Gt: 4,
    ast.CmpOp.GtE: 4,
    ast.CmpOp.Is: 4,
    ast.CmpOp.In: 4,
    ast.CmpOp.Not: 4,
    ast.CmpOp.IsNot: 4,
    ast.CmpOp.NotIn: 4,
    ast.UnaryOp.UAdd: 11,
    ast.UnaryOp.USub: 11,
    ast.UnaryOp.Invert: 11,
    ast.UnaryOp.Not: 3,
}


def precedence(op: Union[ast.BinOp, ast.AugOp, ast.UnaryOp, ast.CmpOp]) -> int:
    """Return KCL operator precedence"""
    if not op:
        return int(OpPrecedence.LOWEST)

    # x ** y -> 12
    if op == ast.BinOp.Pow:
        return int(OpPrecedence.HIGHEST)

    # +x, -x, ~x -> 11
    if op == ast.UnaryOp.UAdd or op == ast.UnaryOp.USub or op == ast.UnaryOp.Invert:
        return OpPrecedence.HIGHEST - 1

    # x * y, x / y, x // y, x % y -> 10
    if (
        op == ast.BinOp.Mul
        or op == ast.BinOp.Div
        or op == ast.BinOp.FloorDiv
        or op == ast.BinOp.Mod
    ):
        return OpPrecedence.HIGHEST - 2

    # x + y, x - y -> 9
    if op == ast.BinOp.Add or op == ast.BinOp.Sub:
        return OpPrecedence.HIGHEST - 3

    # x >> y, x << y -> 8
    if op == ast.BinOp.LShift or op == ast.BinOp.RShift:
        return OpPrecedence.HIGHEST - 4

    # x & y -> 7
    if op == ast.BinOp.BitAnd:
        return OpPrecedence.HIGHEST - 5

    # x | y -> 6
    if op == ast.BinOp.BitOr:
        return OpPrecedence.HIGHEST - 6

    # x ^ y -> 5
    if op == ast.BinOp.BitXor:
        return OpPrecedence.HIGHEST - 7

    # x > y, x < y, etc. -> 4
    if isinstance(op, ast.CmpOp):
        return OpPrecedence.HIGHEST - 8

    # not x -> 3
    if op == ast.UnaryOp.Not:
        return OpPrecedence.HIGHEST - 9

    # x and y -> 2
    if op == ast.BinOp.And:
        return OpPrecedence.HIGHEST - 10

    # x or y -> 1
    if op == ast.BinOp.Or:
        return OpPrecedence.HIGHEST - 11

    # x = y, x += y, x -= y -> 0
    if isinstance(op, ast.AugOp):
        assert OpPrecedence.HIGHEST - 12 == OpPrecedence.LOWEST
        return OpPrecedence.HIGHEST - 12

    return int(OpPrecedence.LOWEST)
