# Copyright 2020 The KCL Authors. All rights reserved.

from typing import cast, List

import kclvm.api.object as objpkg
import kclvm.unification.subsume as subsume

# ------------------------------------
# Type alias
# ------------------------------------

Type = objpkg.KCLBaseTypeObject

# ------------------------------------
# Type annotation str constants
# ------------------------------------

ANY_TYPE_STR = "any"
INT_TYPE_STR = "int"
FLOAT_TYPE_STR = "float"
STR_TYPE_STR = "str"
BOOL_TYPE_STR = "bool"
ITERABLE_TYPE_STR = "str|{:}|[]"
NUMBER_TYPE_STR = "int|float|bool"
NUM_OR_STR_TYPE_STR = "int|float|bool|str"
RESERVED_TYPE_IDENTIFIERS = [
    ANY_TYPE_STR,
    INT_TYPE_STR,
    FLOAT_TYPE_STR,
    STR_TYPE_STR,
    BOOL_TYPE_STR,
]

# ------------------------------------
# Type constants
# ------------------------------------

VOID_TYPE: Type = objpkg.KCLVoidTypeObject()
NONE_TYPE: Type = objpkg.KCLNoneTypeObject()
INT_TYPE: Type = objpkg.KCLIntTypeObject()
FLOAT_TYPE: Type = objpkg.KCLFloatTypeObject()
STR_TYPE: Type = objpkg.KCLStringTypeObject()
BOOL_TYPE: Type = objpkg.KCLBoolTypeObject()
ANY_TYPE: Type = objpkg.KCLAnyTypeObject()
TRUE_LIT_TYPE: Type = objpkg.KCLBoolLitTypeObject(value=True)
FALSE_LIT_TYPE: Type = objpkg.KCLBoolLitTypeObject(value=False)
NUMBER_TYPE: Type = objpkg.KCLUnionTypeObject(types=[INT_TYPE, FLOAT_TYPE, BOOL_TYPE])
NUM_OR_STR_TYPE: Type = objpkg.KCLUnionTypeObject(
    types=[INT_TYPE, FLOAT_TYPE, BOOL_TYPE, STR_TYPE]
)
LIST_ANY_TYPE: Type = objpkg.KCLListTypeObject(item_type=ANY_TYPE)
LIST_STR_TYPE: Type = objpkg.KCLListTypeObject(item_type=STR_TYPE)
DICT_ANY_ANY_TYPE: Type = objpkg.KCLDictTypeObject(
    key_type=ANY_TYPE, value_type=ANY_TYPE
)
DICT_STR_ANY_TYPE: Type = objpkg.KCLDictTypeObject(
    key_type=STR_TYPE, value_type=ANY_TYPE
)
DICT_STR_STR_TYPE: Type = objpkg.KCLDictTypeObject(
    key_type=STR_TYPE, value_type=STR_TYPE
)
INT_OR_STR_TYPE: Type = objpkg.KCLUnionTypeObject(types=[INT_TYPE, STR_TYPE])
ITERABLE_TYPE: Type = objpkg.KCLUnionTypeObject(
    types=[LIST_ANY_TYPE, DICT_ANY_ANY_TYPE, STR_TYPE]
)
LIT_TYPE_KIND_MAPPING = {
    objpkg.KCLTypeKind.StrLitKind: STR_TYPE,
    objpkg.KCLTypeKind.IntLitKind: INT_TYPE,
    objpkg.KCLTypeKind.FloatLitKind: FLOAT_TYPE,
    objpkg.KCLTypeKind.BoolLitKind: BOOL_TYPE,
}

# ------------------------------------
# Type kind constants
# ------------------------------------

NUMBER_TYPE_KINDS = [
    objpkg.KCLTypeKind.IntKind,
    objpkg.KCLTypeKind.FloatKind,
    objpkg.KCLTypeKind.IntLitKind,
    objpkg.KCLTypeKind.FloatLitKind,
    # Please note that the True/False name constant can be used to 1 + True
    objpkg.KCLTypeKind.BoolKind,
    objpkg.KCLTypeKind.BoolLitKind,
]
STR_KINDS = [
    objpkg.KCLTypeKind.StrKind,
    objpkg.KCLTypeKind.StrLitKind,
]
INT_KINDS = [
    objpkg.KCLTypeKind.IntKind,
    objpkg.KCLTypeKind.IntLitKind,
]
FLOAT_KINDS = [
    objpkg.KCLTypeKind.FloatKind,
    objpkg.KCLTypeKind.FloatLitKind,
]
BOOL_KINDS = [
    objpkg.KCLTypeKind.BoolKind,
    objpkg.KCLTypeKind.BoolLitKind,
]
ITERABLE_KINDS = [
    objpkg.KCLTypeKind.ListKind,
    objpkg.KCLTypeKind.DictKind,
    objpkg.KCLTypeKind.SchemaKind,
    objpkg.KCLTypeKind.StrKind,
    objpkg.KCLTypeKind.StrLitKind,
]
KEY_KINDS = STR_KINDS
BUILTIN_KINDS = NUMBER_TYPE_KINDS + STR_KINDS

# -----------------------
# Type functions
# -----------------------


def sup(types: List[Type]) -> Type:
    """The sup function returns the minimum supremum of all types in an array of types"""
    return typeof(types, should_remove_sub_types=True)


def typeof(types: List[Type], should_remove_sub_types: bool = False) -> Type:
    """Build a sup type from types [T1, T2, ... , Tn]"""
    assert isinstance(types, list)
    # 1. Initialize an ordered set to store the type array
    type_set = []
    # 2. Add the type array to the ordered set for sorting by the type id and de-duplication
    add_types_to_type_set(type_set, types)
    # 3. Remove sub types according to partial order relation rules e.g. sub schema types
    if should_remove_sub_types:
        type_set = remove_sub_types(type_set)
    if len(type_set) == 0:
        return ANY_TYPE
    if len(type_set) == 1:
        return type_set[0]
    type_set.sort(key=lambda t: t.type_kind())
    return objpkg.KCLUnionTypeObject(types=type_set)


def add_types_to_type_set(type_set: List[Type], types: List[Type]):
    """Add types into the type set"""
    for tpe in types or []:
        add_type_to_type_set(type_set, tpe)


def add_type_to_type_set(type_set: List[Type], tpe: Type):
    """Add a type into the type set"""
    if isinstance(tpe, objpkg.KCLUnionTypeObject):
        tpe = cast(objpkg.KCLUnionTypeObject, tpe)
        add_types_to_type_set(type_set, tpe.types)
    # Ignore 'void' types in unions
    elif not isinstance(tpe, objpkg.KCLVoidTypeObject):
        if tpe not in type_set:
            type_set.append(tpe)


def remove_sub_types(type_set: List[Type]) -> List[Type]:
    """Remove sub types from the type set"""
    remove_index_list = set()
    for i, source in enumerate(type_set):
        for j, target in enumerate(type_set):
            if i != j:
                is_subsume = subsume.type_subsume(source, target)
                if is_subsume:
                    remove_index_list.add(i)
    return [tpe for i, tpe in enumerate(type_set) if i not in remove_index_list]


def assignable_to(tpe: Type, expected_type: Type) -> bool:
    """Judge type `tpe` can be assigned the expected type"""
    if tpe.type_kind() >= objpkg.KCLTypeKind.VoidKind:
        return False
    if (
        tpe.type_kind()
        == expected_type.type_kind()
        == objpkg.KCLTypeKind.NumberMultiplierKind
    ):
        if expected_type.is_literal():
            return expected_type.type_str() == tpe.type_str()
        else:
            return True
    return subsume.type_subsume(tpe, expected_type, check_left_any=True)


def is_upper_bound(type1: Type, type2: Type) -> bool:
    """Whether `type1` is the upper bound of the `type2`"""
    return subsume.type_subsume(type2, type1)


def has_any_type(types: List[Type]):
    """Whether a type array contains any type"""
    return any([t == ANY_TYPE for t in types])


def infer_to_variable_type(tpe: Type):
    """Infer the value type to the variable type"""
    if tpe is None:
        return tpe
    # Literal type to its named type e.g., 1 -> int, "s" -> str
    if tpe.type_kind() in LIT_TYPE_KIND_MAPPING:
        return LIT_TYPE_KIND_MAPPING[tpe.type_kind()]
    # Union type e.g., 1|2|"s" -> int|str
    if tpe.type_kind() == objpkg.KCLTypeKind.UnionKind:
        return sup([infer_to_variable_type(t) for t in tpe.types])
    # List type e.g., [1|2] -> [int]
    if tpe.type_kind() == objpkg.KCLTypeKind.ListKind:
        tpe.item_type = infer_to_variable_type(tpe.item_type)
    # Dict type e.g., {str:1|2} -> {str:int}
    if tpe.type_kind() == objpkg.KCLTypeKind.DictKind:
        tpe.key_type = infer_to_variable_type(tpe.key_type)
        tpe.value_type = infer_to_variable_type(tpe.value_type)
    # None/Undefined type to any type e.g., None -> any
    if tpe == NONE_TYPE:
        return ANY_TYPE
    return tpe


def literal_union_type_to_variable_type(tpe: Type):
    """Convert the literal union type to its variable type
    e.g., 1|2 -> int, 's'|'ss' -> str.
    """
    if tpe.type_kind() == objpkg.KCLTypeKind.UnionKind:
        return infer_to_variable_type(tpe)
    return tpe


def is_kind_type_or_kind_union_type(tpe: Type, kind_list: List[int]):
    """Judge a type kind in the type kind list or the union
    type kinds are all in the type kind.
    """
    result = False
    if tpe.type_kind() == objpkg.KCLTypeKind.UnionKind:
        result = all([t.type_kind() in kind_list for t in tpe.types])
    elif tpe.type_kind() in kind_list:
        result = True
    return result


def type_to_kcl_type_annotation_str(tpe: Type) -> str:
    """Convert type to a kcl type annotation string"""
    if not tpe or not isinstance(tpe, Type):
        raise ValueError(f"Parameter type must be a valid type, got {tpe}")
    if isinstance(tpe, objpkg.KCLUnionTypeObject):
        return "|".join([type_to_kcl_type_annotation_str(t) for t in tpe.types])
    elif isinstance(tpe, objpkg.KCLIntTypeObject):
        return "int"
    elif isinstance(tpe, objpkg.KCLFloatTypeObject):
        return "float"
    elif isinstance(tpe, objpkg.KCLStringTypeObject):
        return "str"
    elif isinstance(tpe, objpkg.KCLBoolTypeObject):
        return "bool"
    elif isinstance(tpe, objpkg.KCLAnyTypeObject):
        return "any"
    elif isinstance(tpe, objpkg.KCLStringLitTypeObject):
        return '"{}"'.format(tpe.value.replace('"', '\\"'))
    elif isinstance(
        tpe,
        (
            objpkg.KCLIntLitTypeObject,
            objpkg.KCLFloatLitTypeObject,
            objpkg.KCLBoolLitTypeObject,
        ),
    ):
        return str(tpe.value)
    elif isinstance(tpe, objpkg.KCLListTypeObject):
        return "[" + type_to_kcl_type_annotation_str(tpe.item_type) + "]"
    elif isinstance(tpe, objpkg.KCLDictTypeObject):
        return (
            "{"
            + type_to_kcl_type_annotation_str(tpe.key_type)
            + ":"
            + type_to_kcl_type_annotation_str(tpe.value_type)
            + "}"
        )
    elif isinstance(tpe, objpkg.KCLSchemaDefTypeObject):
        return tpe.schema_type.type_str_with_pkgpath()
    elif isinstance(tpe, objpkg.KCLSchemaTypeObject):
        return tpe.type_str_with_pkgpath()
    elif isinstance(tpe, objpkg.KCLNumberMultiplierTypeObject):
        return (
            f"{tpe.raw_value}{tpe.binary_suffix}"
            if tpe.is_literal()
            else "units.NumberMultiplier"
        )
    return ""
