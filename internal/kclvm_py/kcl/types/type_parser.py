"""The `type_parser` file mainly contains the function `parse_type_str`
which is used to parser a type string to a KCL type object

:copyright: Copyright 2020 The KCL Authors. All rights reserved.
"""

import re as _re

from typing import Dict
from ast import literal_eval

import kclvm.api.object as objpkg
import kclvm.api.object.internal.common as common
import kclvm.compiler.build.utils.units as units

from .type import (
    Type,
    INT_TYPE,
    FLOAT_TYPE,
    STR_TYPE,
    BOOL_TYPE,
    ANY_TYPE,
    TRUE_LIT_TYPE,
    FALSE_LIT_TYPE,
    DICT_STR_ANY_TYPE,
    DICT_STR_STR_TYPE,
    sup,
)

BUILTIN_TYPES = ["str", "bool", "int", "float"]
_KCL_TYPE_any = "any"
_KCL_TYPE_True = "True"
_KCL_TYPE_False = "False"


TYPES_MAPPING: Dict[str, Type] = {
    "int": INT_TYPE,
    "float": FLOAT_TYPE,
    "str": STR_TYPE,
    "bool": BOOL_TYPE,
    "any": ANY_TYPE,
    "[]": objpkg.KCLListTypeObject(item_type=ANY_TYPE),
    "[any]": objpkg.KCLListTypeObject(item_type=ANY_TYPE),
    "[str]": objpkg.KCLListTypeObject(item_type=STR_TYPE),
    "{:}": objpkg.KCLDictTypeObject(key_type=ANY_TYPE, value_type=ANY_TYPE),
    "{str:}": DICT_STR_ANY_TYPE,
    "{str:any}": DICT_STR_ANY_TYPE,
    "{str:str}": DICT_STR_STR_TYPE,
}


def parse_type_str(tpe_str: str) -> Type:
    """Parser a type string to a type object"""
    if tpe_str is None or tpe_str == "":
        return ANY_TYPE
    if not isinstance(tpe_str, str):
        raise ValueError(f"Argument tpe_str must be str, not {type(tpe_str)}")
    # Remove the space in the type string
    tpe_str = tpe_str.strip(" \t\f\v\r\n")
    if tpe_str in TYPES_MAPPING:
        return TYPES_MAPPING[tpe_str]
    # Union type
    if is_union_type_str(tpe_str):
        return parse_union_type_str(tpe_str)
    # Bultin literal type
    elif is_lit_type_str(tpe_str):
        return parse_lit_type_str(tpe_str)
    # Number multiplier literal type
    elif is_number_multiplier_literal_type(tpe_str):
        return parse_number_multiplier_literal_type(tpe_str)
    # Dict type
    elif is_dict_type_str(tpe_str):
        k_type_str, v_type_str = common.separate_kv(common.dereferencetype(tpe_str))
        return objpkg.KCLDictTypeObject(
            key_type=parse_type_str(k_type_str),
            value_type=parse_type_str(v_type_str),
        )
    # List type
    elif is_list_type_str(tpe_str):
        return objpkg.KCLListTypeObject(
            item_type=parse_type_str(common.dereferencetype(tpe_str))
        )
    # Schema type or pkg.Schema type or named type
    return parse_named_type(tpe_str)


# -----------------------
# Judge type string
# -----------------------


def is_union_type_str(tpe: str) -> bool:
    """Whether a type string is a union type string, e.g. A|B|C,
    and detect '|' in type string except '|' in dict or list.
    """
    return common.is_type_union(tpe)


def is_list_type_str(tpe: str) -> bool:
    """Whether a type string is a list type string"""
    return common.islisttype(tpe)


def is_dict_type_str(tpe: str) -> bool:
    """Whether a type string is a dict type string"""
    return common.isdicttype(tpe)


def is_lit_type_str(tpe: str) -> bool:
    """Whether a type string is a literal type string"""
    if tpe in [_KCL_TYPE_True, _KCL_TYPE_False]:
        return True

    # str
    if tpe.startswith('"'):
        return tpe.endswith('"')
    if tpe.startswith("'"):
        return tpe.endswith("'")

    # int or float
    try:
        float(tpe)
        return True
    except ValueError:
        pass

    # non literal type
    return False


def is_number_multiplier_literal_type(tpe: str) -> bool:
    """Whether a type string is a number multiplier literal type string"""
    return bool(_re.match(units.NUMBER_MULTIPLIER_REGEX, tpe))


# -----------------------
# Parse type string
# -----------------------


def parse_union_type_str(tpe_str: str) -> Type:
    """Parse union type string"""
    type_str_list = common.split_type_union(tpe_str)
    types = [parse_type_str(tpe) for tpe in type_str_list]
    return sup(types)


def parse_lit_type_str(tpe_str: str) -> Type:
    """Parse literal type string"""
    type_val = literal_eval(tpe_str)
    if isinstance(type_val, bool):
        return TRUE_LIT_TYPE if type_val else FALSE_LIT_TYPE
    if isinstance(type_val, str):
        return objpkg.KCLStringLitTypeObject(value=type_val)
    if isinstance(type_val, int):
        return objpkg.KCLIntLitTypeObject(value=type_val)
    if isinstance(type_val, float):
        return objpkg.KCLFloatLitTypeObject(value=type_val)
    raise ValueError(f"Invalid argument tpe_str {tpe_str}")


def parse_number_multiplier_literal_type(tpe_str: str) -> Type:
    """Parse number multiplier literal type"""
    if tpe_str[-1] == units.IEC_SUFFIX:
        value, suffix = int(tpe_str[:-2]), tpe_str[-2:]
    else:
        value, suffix = int(tpe_str[:-1]), tpe_str[-1]
    return objpkg.KCLNumberMultiplierTypeObject(
        value=units.cal_num(value, suffix),
        raw_value=value,
        binary_suffix=suffix,
    )


def parse_named_type(tpe_str: str) -> Type:
    # Please note Named type to find it in the scope (e.g. schema type, type alias)
    return objpkg.KCLNamedTypeObject(name=tpe_str)


# -----------------------
# END
# -----------------------
