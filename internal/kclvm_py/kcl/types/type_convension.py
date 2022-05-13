# Copyright 2020 The KCL Authors. All rights reserved.

import kclvm.kcl.error as kcl_error
import kclvm.api.object as objpkg

from .type import Type


def type_convert(obj: objpkg.KCLObject, tpe: Type) -> objpkg.KCLObject:
    """The type of `obj` is converted to another type.

    Raise an runtime error occurs in the type conversion.
    """
    if not obj or not isinstance(obj, objpkg.KCLObject):
        raise ValueError("Invalid parameter obj, expected KCL object")
    if not tpe or not isinstance(tpe, Type):
        raise ValueError("Invalid parameter tpe, expected KCL type object")
    if isinstance(obj, (objpkg.KCLNoneObject, objpkg.KCLUndefinedObject)):
        return obj
    if isinstance(tpe, objpkg.KCLAnyTypeObject):
        return obj
    if isinstance(tpe, objpkg.KCLIntTypeObject) and isinstance(
        obj, (objpkg.KCLIntObject, objpkg.KCLFloatObject)
    ):
        return objpkg.KCLIntObject(int(obj.value))
    if isinstance(tpe, objpkg.KCLFloatTypeObject) and isinstance(
        obj, (objpkg.KCLIntObject, objpkg.KCLFloatObject)
    ):
        return objpkg.KCLFloatObject(float(obj.value))
    if isinstance(tpe, objpkg.KCLStringTypeObject) and isinstance(
        obj, objpkg.KCLStringObject
    ):
        return obj
    if isinstance(tpe, objpkg.KCLBoolTypeObject) and isinstance(
        obj, objpkg.KCLNameConstantObject
    ):
        return obj
    if isinstance(tpe, objpkg.KCLListTypeObject) and isinstance(
        obj, objpkg.KCLListObject
    ):
        return objpkg.KCLListObject(
            items=[type_convert(item, tpe.item_type) for item in obj.items]
        )
    if isinstance(tpe, objpkg.KCLDictTypeObject) and isinstance(
        obj, objpkg.KCLDictObject
    ):
        if isinstance(obj, objpkg.KCLSchemaConfigObject):
            return objpkg.KCLSchemaConfigObject(
                operation_map=obj.operation_map,
                insert_index_map=obj.insert_index_map,
                value={
                    k: type_convert(obj.value[k], tpe.value_type) for k in obj.value
                },
            )
        else:
            return objpkg.KCLDictObject(
                value={k: type_convert(obj.value[k], tpe.value_type) for k in obj.value}
            )
    if (
        isinstance(tpe, objpkg.KCLSchemaTypeObject)
        and isinstance(obj, objpkg.KCLSchemaObject)
        and obj.runtime_type == tpe.runtime_type
    ):
        return obj
    if isinstance(tpe, objpkg.KCLSchemaDefTypeObject):
        return type_convert(obj, tpe.schema_type)
    kcl_error.report_exception(
        err_type=kcl_error.ErrType.EvaluationError_TYPE,
        file_msgs=[kcl_error.ErrFileMsg(filename=None, line_no=None, col_no=None)],
        arg_msg=f"Cannot convert type '{obj.type_str()}' to '{tpe.type_str()}'",
    )
