# Copyright 2021 The KCL Authors. All rights reserved.

from typing import cast

import kclvm.api.object as obj


def value_subsume(
    value1: obj.KCLObject, value2: obj.KCLObject, should_recursive_check: bool = True
) -> bool:
    """Calculate the partial order relationship between `KCL value objects`
    and judge whether the value1 object ∈ the value2 object.

    Please note that The type and value of KCL are defined and used separately,
    so the partial order relationship calculation is also divided into two types,
    type and value, and there is no partial order relationship between type
    objects and value objects.
    """
    if not value1 or not value2:
        return False
    if not isinstance(value1, obj.KCLObject) or not isinstance(value2, obj.KCLObject):
        return False
    if value1 == value2 or value1 is value2:
        return True
    if isinstance(value1, (obj.KCLNoneObject, obj.KCLUndefinedObject)):
        return True
    if isinstance(value2, (obj.KCLNoneObject, obj.KCLUndefinedObject)):
        return True
    if isinstance(value1, obj.KCLIntObject):
        return isinstance(value2, obj.KCLIntObject) and value1.value == value2.value
    if isinstance(value1, obj.KCLFloatObject):
        return isinstance(value2, obj.KCLFloatObject) and value1.value == value2.value
    if isinstance(value1, obj.KCLNameConstantObject):
        return (
            isinstance(value2, obj.KCLNameConstantObject)
            and value1.value == value2.value
        )
    if isinstance(value1, (obj.KCLListObject, obj.KCLTupleObject)):
        return (
            isinstance(value2, (obj.KCLListObject, obj.KCLTupleObject))
            and len(value1.items) == len(value2.items)
            and all(
                [
                    value_subsume(item1, item2, should_recursive_check)
                    for item1, item2 in zip(value1.items, value2.items)
                ]
            )
        )
    if isinstance(value1, (obj.KCLDictObject, obj.KCLSchemaObject)):
        if isinstance(value2, (obj.KCLDictObject, obj.KCLSchemaObject)):
            value1_dict = {k: value1.get(k) for k in sorted(list(value1.value.keys()))}
            value2_dict = {k: value2.get(k) for k in sorted(list(value2.value.keys()))}

            if len(value1_dict) == 0:
                return True

            if all([key not in value2_dict for key in value1_dict]):
                return True

            if should_recursive_check:
                for key1, value1 in value1_dict.items():
                    if key1 not in value2_dict:
                        continue
                    value2 = value2_dict.get(key1)
                    if not value_subsume(value1, value2, should_recursive_check):
                        return False
            return True
    return False


def type_subsume(
    value1: obj.KCLObject, value2: obj.KCLObject, check_left_any: bool = False
) -> bool:
    """Calculate the partial order relationship between `KCL type objects`
    and judge whether the value1 object ∈ the value2 object.

    Please note that The type and value of KCL are defined and used separately,
    so the partial order relationship calculation is also divided into two types,
    type and value, and there is no partial order relationship between type
    objects and value objects.
    """
    if not value1 or not value2:
        return False
    if not isinstance(value1, obj.KCLObject) or not isinstance(value2, obj.KCLObject):
        return False
    if value1 == value2 or value1 is value2:
        return True
    if check_left_any and isinstance(value1, obj.KCLAnyTypeObject):
        return True
    if isinstance(value2, obj.KCLAnyTypeObject):
        return True
    if isinstance(value1, obj.KCLNoneTypeObject):
        return True
    if isinstance(value1, obj.KCLUnionTypeObject):
        return all([type_subsume(tpe, value2) for tpe in value1.types])
    if isinstance(value2, obj.KCLUnionTypeObject):
        return any([type_subsume(value1, tpe) for tpe in value2.types])
    if isinstance(value1, obj.KCLSchemaTypeObject):
        if not isinstance(value2, obj.KCLSchemaTypeObject):
            return False
        value1 = cast(obj.KCLSchemaTypeObject, value1)
        value2 = cast(obj.KCLSchemaTypeObject, value2)
        return value1.is_sub_schema_of(value2)
    if isinstance(value1, obj.KCLIntTypeObject) and isinstance(
        value2, obj.KCLFloatTypeObject
    ):
        return True
    if isinstance(value1, obj.KCLBuiltinTypeObject):
        return (
            isinstance(value2, obj.KCLBuiltinTypeObject)
            and value1.type_kind() == value2.type_kind()
        )
    if isinstance(
        value1,
        (
            obj.KCLStringLitTypeObject,
            obj.KCLNumberLitTypeObject,
            obj.KCLBoolLitTypeObject,
        ),
    ):
        if isinstance(
            value2,
            (
                obj.KCLStringLitTypeObject,
                obj.KCLNumberLitTypeObject,
                obj.KCLBoolLitTypeObject,
            ),
        ):
            return (
                value1.type_kind() == value2.type_kind()
                and value1.value == value2.value
            )
        elif isinstance(value2, obj.KCLBuiltinTypeObject):
            # float_lit -> float
            # int_lit -> int
            # bool_lit -> bool
            # str_lit -> str
            # int_lit/bool_lit -> float
            if isinstance(value2, obj.KCLFloatTypeObject) and not isinstance(
                value1, obj.KCLStringLitTypeObject
            ):
                return True
            return value2.type_str() in value1.type_str()
    if isinstance(value1, obj.KCLListTypeObject):
        return isinstance(value2, obj.KCLListTypeObject) and type_subsume(
            value1.item_type, value2.item_type, check_left_any
        )
    if isinstance(value1, obj.KCLDictTypeObject):
        return (
            isinstance(value2, obj.KCLDictTypeObject)
            and type_subsume(value1.key_type, value2.key_type, check_left_any)
            and type_subsume(value1.value_type, value2.value_type, check_left_any)
        )
    if isinstance(value1, obj.KCLNumberMultiplierTypeObject) and isinstance(
        value2, obj.KCLNumberMultiplierTypeObject
    ):
        if value1.is_literal():
            return not value2.is_literal() and value1.type_str() == value2.type_str()
        else:
            return True
    return False
