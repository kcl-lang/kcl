# Copyright 2021 The KCL Authors. All rights reserved.

from typing import List, Union, cast

import kclvm.kcl.error as kcl_error
import kclvm.kcl.ast as ast
from kclvm.api.object import (
    KCLObject,
    KCLSchemaObject,
    KCLSchemaTypeObject,
    KCLListObject,
    KCLDictObject,
    KCLNoneObject,
    KCLUndefinedObject,
    KCLStringObject,
    KCLLiteralObject,
    KCLSchemaConfigObject,
    KCLConfigObjectMixin,
    to_python_obj,
    to_kcl_obj,
)
from kclvm.api.object.internal import Undefined, UndefinedType
from kclvm.compiler.check.check_type import check_type_builtin
from kclvm.unification import value_subsume


def override_config_attr(
    attr: str,
    obj: Union[KCLDictObject, KCLSchemaObject],
    config_value: KCLObject,
    index: int = None,
):
    if index is None or index < 0:
        obj.update_key_value(attr, config_value)
    else:
        obj.list_key_override(attr, config_value, index)


def insert_config_attr(
    attr: str,
    obj: Union[KCLDictObject, KCLSchemaObject],
    config_value: KCLObject,
    index: int = None,
):
    if index is not None and index >= 0:
        config_value = KCLListObject(items=[config_value])
    obj.insert_with_key(attr, config_value, index)


def resolve_schema_obj(
    schema_obj: KCLSchemaObject, keys: set, vm=None
) -> KCLSchemaObject:
    """Using a schema object config to resolve and generate a new schema"""
    if not vm or not schema_obj or not isinstance(schema_obj, KCLSchemaObject):
        return schema_obj
    schema_type_obj = cast(
        KCLSchemaTypeObject,
        vm.all_schema_types.get(f"{schema_obj.pkgpath}.{schema_obj.name}"),
    )
    if not schema_type_obj:
        return schema_obj
    filename, line, column = vm.get_info()
    config_meta = {
        "$filename": filename,
        "$lineno": line,
        "$columnno": column,
    }
    config = KCLSchemaConfigObject(
        value={k: schema_obj.attrs[k] for k in schema_obj.attrs if k in keys},
        operation_map=schema_obj.operation_map,
    )
    return schema_type_obj.new_instance(config, config_meta, [], [], vm)


def do_union(
    obj: KCLObject,
    delta: KCLObject,
    should_list_override: bool = False,
    should_idempotent_check: bool = False,
    should_config_resolve: bool = False,
    vm=None,
) -> KCLObject:
    """
    Union delta to obj recursively
    """
    obj_tpe = obj.type_str()
    delta_tpe = delta.type_str()
    if isinstance(delta, KCLStringObject):
        delta_tpe = "str"
    if isinstance(obj, KCLListObject):
        if not isinstance(delta, KCLListObject):
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.TypeError_Runtime_TYPE,
                arg_msg=f"union failure, expect list, got {delta_tpe}",
            )

        length = (
            len(obj.value) if len(obj.value) > len(delta.value) else len(delta.value)
        )
        if should_list_override:
            return delta
        result_list = obj
        for idx in range(length):
            if idx >= len(obj.value):
                result_list.value.append(delta.value[idx])
            elif idx < len(delta.value):
                result_list.value[idx] = union(
                    result_list.value[idx],
                    delta.value[idx],
                    should_list_override=should_list_override,
                    should_idempotent_check=should_idempotent_check,
                    should_config_resolve=should_config_resolve,
                    vm=vm,
                )
        return result_list
    if isinstance(obj, KCLDictObject):
        if not isinstance(delta, KCLDictObject):
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.TypeError_Runtime_TYPE,
                arg_msg=f"union failure, expect dict, got {delta_tpe}",
            )
        result_dict = obj
        if isinstance(obj, KCLConfigObjectMixin):
            obj.update_attr_op_using_obj(delta)
        for k in delta.value:
            operation = (
                delta.get_operation(k)
                if isinstance(delta, KCLConfigObjectMixin)
                else ast.ConfigEntryOperation.UNION
            )
            insert_index = (
                delta.get_insert_index(k)
                if isinstance(delta, KCLConfigObjectMixin)
                else None
            )
            if k not in obj.value:
                result_dict.value[k] = delta.value[k]
            else:
                if operation == ast.ConfigEntryOperation.OVERRIDE:
                    override_config_attr(k, result_dict, delta.value[k], insert_index)
                if operation == ast.ConfigEntryOperation.INSERT:
                    insert_config_attr(k, result_dict, delta.value[k], insert_index)
                else:
                    if (
                        should_idempotent_check
                        and k in obj.value
                        and not value_subsume(delta.value[k], obj.value[k], False)
                    ):
                        kcl_error.report_exception(
                            err_type=kcl_error.ErrType.EvaluationError_TYPE,
                            arg_msg=f"conflicting values on the attribute '{k}' between "
                            f"{to_python_obj(delta)} and {to_python_obj(obj)}",
                        )
                    result_dict.value[k] = union(
                        obj.value[k],
                        delta.value[k],
                        should_list_override=should_list_override,
                        should_idempotent_check=should_idempotent_check,
                        should_config_resolve=should_config_resolve,
                        vm=vm,
                    )
        return result_dict
    if isinstance(obj, KCLSchemaObject):
        delta_dict = {}
        if isinstance(delta, (KCLDictObject, KCLSchemaObject)):
            delta_dict = delta.value
        else:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.TypeError_Runtime_TYPE,
                arg_msg=f"union failure, expect {obj_tpe}, got {delta_tpe}",
            )
        if should_config_resolve:
            common_keys = obj.config_keys | delta.config_keys
        if isinstance(obj, KCLConfigObjectMixin):
            obj.update_attr_op_using_obj(delta)
        for k in delta_dict:
            if should_config_resolve and k not in common_keys:
                continue
            operation = (
                delta.get_operation(k)
                if isinstance(delta, KCLConfigObjectMixin)
                else ast.ConfigEntryOperation.UNION
            )
            insert_index = (
                delta.get_insert_index(k)
                if isinstance(delta, KCLConfigObjectMixin)
                else None
            )
            if (
                should_config_resolve
                and k not in obj.attrs
                and not obj.should_add_attr(k)
            ):
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.CannotAddMembers_Runtime_TYPE,
                    arg_msg=kcl_error.CANNOT_ADD_MEMBERS_MSG.format(k, obj.name),
                )
            if operation == ast.ConfigEntryOperation.OVERRIDE:
                override_config_attr(k, obj, delta_dict[k], insert_index)
            if operation == ast.ConfigEntryOperation.INSERT:
                insert_config_attr(k, obj, delta_dict[k], insert_index)
            else:
                if (
                    should_idempotent_check
                    and k in obj.attrs
                    and not value_subsume(delta_dict[k], obj.attrs[k], False)
                ):
                    kcl_error.report_exception(
                        err_type=kcl_error.ErrType.EvaluationError_TYPE,
                        arg_msg=f"conflicting values on the attribute '{k}' between "
                        f"{to_python_obj(obj)} and {to_python_obj(delta_dict)}",
                    )
                obj.attrs[k] = union(
                    obj.attrs.get(k),
                    delta_dict[k],
                    should_list_override=should_list_override,
                    should_idempotent_check=should_idempotent_check,
                    should_config_resolve=should_config_resolve,
                    vm=vm,
                )
            # Do type check and pack
            if isinstance(obj.attrs[k], KCLLiteralObject):
                check_type_builtin(obj.attrs[k], obj.get_attr_type(k))
        if should_config_resolve:
            obj = resolve_schema_obj(obj, common_keys, vm=vm)
            obj.config_keys = common_keys
        return obj
    if type(obj) != type(delta):
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.TypeError_Runtime_TYPE,
            arg_msg=f"union failure, expect {obj_tpe}, got {delta_tpe}",
        )
    return delta


def union(
    obj: KCLObject,
    delta: KCLObject,
    or_mode: bool = False,
    should_list_override: bool = False,
    should_idempotent_check: bool = False,
    should_config_resolve: bool = False,
    vm=None,
) -> KCLObject:
    if (
        obj is None
        or obj is Undefined
        or isinstance(obj, UndefinedType)
        or isinstance(obj, (KCLNoneObject, KCLUndefinedObject))
    ):
        return delta
    if (
        delta is None
        or delta is Undefined
        or isinstance(delta, UndefinedType)
        or isinstance(delta, (KCLNoneObject, KCLUndefinedObject))
    ):
        return obj
    if isinstance(obj, (KCLListObject, KCLSchemaObject, KCLDictObject)) or isinstance(
        delta, (KCLListObject, KCLSchemaObject, KCLDictObject)
    ):
        return do_union(
            obj,
            delta,
            should_list_override=should_list_override,
            should_idempotent_check=should_idempotent_check,
            should_config_resolve=should_config_resolve,
            vm=vm,
        )
    if or_mode:
        return to_kcl_obj(to_python_obj(obj) | to_python_obj(delta))
    else:
        return obj if isinstance(delta, (KCLNoneObject, KCLUndefinedObject)) else delta


def merge(objs: List[KCLObject], vm=None) -> KCLObject:
    """Merge all objects recursively

    - literal variables, override
    - list variables, override
    - dict/schema variables, union
    """
    initial_object = KCLNoneObject.instance()
    if not objs:
        return initial_object
    for obj in objs:
        if not obj or isinstance(obj, (KCLNoneObject, KCLUndefinedObject)):
            continue
        initial_object = union(initial_object, obj, should_list_override=True, vm=vm)
    return initial_object
