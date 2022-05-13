# Copyright 2021 The KCL Authors. All rights reserved.

import re
from ast import literal_eval
from typing import Union

import kclvm.kcl.ast as ast
import kclvm.compiler.parser as parser
import kclvm.api.object.internal as internal


def BuildNodeFromString(value: str, line: int = None, column: int = None) -> ast.Expr:
    lit_node = BuildLitNodeFromString(value, line, column)
    if isinstance(lit_node, ast.StringLit):
        try:
            val = parser.ParseExpr(code=value)
            # If `val` is a identifier, convert it to a string literal
            return lit_node if isinstance(val, ast.Identifier) else val
        except Exception:
            return lit_node
    return lit_node


def BuildLitNodeFromValue(
    value: Union[int, float, str, bool], line: int = None, column: int = None
) -> ast.Literal:
    if value is None:
        val = ast.NameConstantLit()
        val.value = None
    elif value is internal.Undefined:
        val = ast.NameConstantLit()
        val.value = internal.Undefined
    elif value is True:
        val = ast.NameConstantLit()
        val.value = True
    elif value is False:
        val = ast.NameConstantLit()
        val.value = False
    elif isinstance(value, (int, float)):
        val = ast.NumberLit(value=value)
    else:
        val = ast.StringLit()
        val.value = value if isinstance(value, str) else str(value)
    val.line = line
    val.column = column
    return val


def BuildLitNodeFromString(
    value: str, line: int = None, column: int = None
) -> ast.Literal:
    if value in ["True", "true"]:
        val = ast.NameConstantLit()
        val.value = True
    elif value in ["False", "false"]:
        val = ast.NameConstantLit()
        val.value = False
    elif value in ["None", "null"]:
        val = ast.NameConstantLit()
        val.value = None
    elif value in ["Undefined"]:
        val = ast.NameConstantLit()
        val.value = internal.Undefined
    elif is_number(value):
        val = ast.NumberLit(value=literal_eval(value))
    else:
        val = ast.StringLit()
        val.value = str(value)
        val.raw_value = value

        if val.value and val.value[0] == "'" and val.value[-1] == "'":
            val.value = val.value[1:-1]
        elif val.value and val.value[0] == '"' and val.value[-1] == '"':
            val.value = val.value[1:-1]

        if (
            val.raw_value
            and val.raw_value[0] not in ["'", '"']
            and val.raw_value[-1] not in ["'", '"']
        ):
            val.raw_value = '"' + val.raw_value.replace('"', '\\"') + '"'
    val.line = line
    val.column = column
    return val


def is_number(value: str):
    """Whether a string is a number string"""
    pattern = re.compile(r"^[-+]?[-0-9]\d*\.\d*|[-+]?\.?[0-9]\d*$")
    return bool(pattern.match(value))
