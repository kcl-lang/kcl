#! /usr/bin/env python3

from ast import literal_eval
from typing import Optional, List

import kclvm.kcl.error as kcl_error
import kclvm.kcl.info as kcl_info
import kclvm.config
import kclvm.api.object as objpkg
import kclvm.api.object.internal.common as common
from kclvm.api.object.internal import Undefined, UndefinedType

STR_TYPE = "str"
BOOL_TYPE = "bool"
INT_TYPE = "int"
FLOAT_TYPE = "float"
BUILTIN_TYPES = [STR_TYPE, BOOL_TYPE, INT_TYPE, FLOAT_TYPE]

_KCL_TYPE_any = "any"
_KCL_TYPE_True = "True"
_KCL_TYPE_False = "False"
_KCL_TYPE_None = "None"


# ------------------------------
# Numeric range check
# ------------------------------


def check(
    kcl_obj,
    filename: Optional[str] = None,
    lineno: Optional[int] = None,
    columnno: Optional[int] = None,
):
    """Check whether the KCL object meets the scope requirements"""
    if not kclvm.config.debug:
        return kcl_obj
    strict_range_check = kclvm.config.strict_range_check
    check_bit = 32 if strict_range_check else 64
    int_min = kcl_info.INT32_MIN if strict_range_check else kcl_info.INT64_MIN
    int_max = kcl_info.INT32_MAX if strict_range_check else kcl_info.INT64_MAX
    float_min = kcl_info.FLOAT32_MIN if strict_range_check else kcl_info.FLOAT64_MIN
    float_max = kcl_info.FLOAT32_MAX if strict_range_check else kcl_info.FLOAT64_MAX

    def check_object(obj: objpkg.KCLObject):
        if isinstance(obj, objpkg.KCLIntObject):
            value = obj.value
            if not (int_min <= value <= int_max):
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.IntOverflow_TYPE,
                    file_msgs=[
                        kcl_error.ErrFileMsg(
                            filename=filename, line_no=lineno, col_no=columnno
                        )
                    ],
                    arg_msg=kcl_error.INT_OVER_FLOW_MSG.format(
                        str(obj.value), check_bit
                    ),
                )
        elif isinstance(obj, objpkg.KCLFloatObject):
            abs_var = abs(obj.value)
            if 0 < abs_var < float_min:
                kcl_error.report_warning(
                    err_type=kcl_error.ErrType.FloatUnderflow_TYPE,
                    file_msgs=[
                        kcl_error.ErrFileMsg(
                            filename=filename, line_no=lineno, col_no=columnno
                        )
                    ],
                    arg_msg=kcl_error.FLOAT_UNDER_FLOW_MSG.format(
                        str(obj.value), check_bit
                    ),
                )
                obj.value = 0.0
            elif abs_var > float_max:
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.FloatOverflow_TYPE,
                    file_msgs=[
                        kcl_error.ErrFileMsg(
                            filename=filename, line_no=lineno, col_no=columnno
                        )
                    ],
                    arg_msg=kcl_error.FLOAT_OVER_FLOW_MSG.format(
                        str(obj.value), check_bit
                    ),
                )
        elif isinstance(obj, objpkg.KCLListObject):
            for i in obj.value:
                check_object(i)
        elif isinstance(obj, (objpkg.KCLDictObject, objpkg.KCLSchemaObject)):
            for k, v in obj.value.items():
                check_object(k)
                check_object(v)
        return obj

    obj = check_object(kcl_obj)
    return obj


# ------------------------------
# Type pack and check functions
# ------------------------------


def runtime_types(
    tpes: List[str],
    vm=None,
    filename: Optional[str] = None,
    lineno: Optional[int] = None,
    columnno: Optional[int] = None,
):
    if not tpes:
        return tpes
    runtime_tpes = []
    for tpe in tpes:
        runtime_tpe = runtime_type(tpe, vm, filename, lineno, columnno)
        if runtime_tpe:
            runtime_tpes.append(runtime_tpe)
    return runtime_tpes


def runtime_type(
    tpe: str,
    vm=None,
    filename: Optional[str] = None,
    lineno: Optional[int] = None,
    columnno: Optional[int] = None,
):
    if not tpe:
        return tpe
    if common.isdicttype(tpe):
        tpe = common.dereferencetype(tpe)
        key_type, value_type = common.separate_kv(tpe)
        if key_type is None or value_type is None:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.TypeError_Runtime_TYPE,
                file_msgs=[
                    kcl_error.ErrFileMsg(
                        filename=filename, line_no=lineno, col_no=columnno
                    )
                ],
                arg_msg="error in dict type, key-value type can't be None",
            )
        key_type = runtime_type(
            key_type,
            vm,
            filename,
            lineno,
            columnno,
        )
        value_type = runtime_type(
            value_type,
            vm,
            filename,
            lineno,
            columnno,
        )
        return "{{{}:{}}}".format(key_type, value_type)
    elif common.islisttype(tpe):
        tpe = common.dereferencetype(tpe)
        ele_type = runtime_type(tpe, vm, filename, lineno, columnno)
        return "[{}]".format(ele_type)
    elif tpe in BUILTIN_TYPES:
        return tpe
    else:
        if "." in tpe:
            schema_tpe_obj = vm.find_schema_type(tpe)
            if isinstance(schema_tpe_obj, objpkg.KCLSchemaTypeObject):
                return schema_tpe_obj.runtime_type
        else:
            schema_name = tpe
            if schema_name in vm.ctx.globals and isinstance(
                vm.ctx.globals[schema_name], objpkg.KCLSchemaTypeObject
            ):
                return vm.ctx.globals[schema_name].runtime_type
    return None


def convert_collection_value(
    value,
    expected_type: Optional[str],
    filename: Optional[str] = None,
    lineno: Optional[int] = None,
    columnno: Optional[int] = None,
    vm=None,
    config_meta=None,
):
    assert isinstance(value, objpkg.KCLObject)

    if expected_type == _KCL_TYPE_any:
        return value

    is_collection = isinstance(value, (objpkg.KCLDictObject, objpkg.KCLListObject))
    invalid_match_dict = common.isdicttype(expected_type) and not isinstance(
        value, objpkg.KCLDictObject
    )
    invalid_match_list = common.islisttype(expected_type) and not isinstance(
        value, objpkg.KCLListObject
    )
    invalid_match = invalid_match_dict or invalid_match_list
    if (
        not expected_type
        or not is_collection
        or invalid_match
        or common.is_type_union(expected_type)
    ):
        return value
    if common.isdicttype(expected_type):
        # convert dict
        key_tpe, value_tpe = common.separate_kv(common.dereferencetype(expected_type))
        expected_dict = {}
        for k, v in value.value.items():
            expected_value = convert_collection_value(
                v, value_tpe, filename, lineno, columnno, vm
            )
            expected_dict[k] = expected_value
        obj = objpkg.KCLSchemaConfigObject(value=expected_dict)
        obj.update_attr_op_using_obj(value)
        return obj
    elif common.islisttype(expected_type):
        # convert list
        expected_type = common.dereferencetype(expected_type)
        expected_list = []
        for i in value.value:
            expected_schema = convert_collection_value(
                i,
                expected_type,
                filename,
                lineno,
                columnno,
                vm,
            )
            expected_list.append(expected_schema)
        return objpkg.KCLListObject(expected_list)
    elif expected_type in BUILTIN_TYPES:
        # Do nothing on built-in types
        return value
    elif isinstance(value, objpkg.KCLListObject):
        # List value not match the schema type
        return value
    elif "." in expected_type:
        # use cross pkg schema like 'pkg.schema'
        # kcl support use cross pkg schema without import
        schema_type = vm.find_schema_type(expected_type)
        if schema_type and isinstance(schema_type, objpkg.KCLSchemaTypeObject):
            config_meta_new = config_meta or vm.ctx.locals.get(
                objpkg.SCHEMA_CONFIG_META_KEY
            )
            return schema_type.new_instance(value, config_meta_new, [], [], vm)
    elif (
        expected_type in vm.ctx.globals
        and vm.ctx.globals[expected_type].type() == objpkg.KCLObjectType.SCHEMA_TYPE
    ):
        # Schema in current module context without import
        config_meta_new = config_meta or vm.ctx.locals.get(
            objpkg.SCHEMA_CONFIG_META_KEY
        )
        schema_type = vm.ctx.globals[expected_type]
        return schema_type.new_instance(value, config_meta_new, [], [], vm)
    kcl_error.report_exception(
        err_type=kcl_error.ErrType.EvaluationError_TYPE,
        file_msgs=[
            kcl_error.ErrFileMsg(filename=filename, line_no=lineno, col_no=columnno)
        ],
        arg_msg="name '{}' is not defined".format(expected_type),
    )


def type_pack_and_check(
    value,
    expected_types: List[str],
    filename=None,
    lineno=None,
    columno=None,
    vm=None,
    config_meta=None,
):
    """
    Type pack and check
    """
    if value is None or value is Undefined or isinstance(value, UndefinedType):
        return value
    value = objpkg.to_kcl_obj(value)
    if (
        isinstance(value, (objpkg.KCLNoneObject, objpkg.KCLUndefinedObject))
        or not expected_types
    ):
        return value
    is_schema = isinstance(value, objpkg.KCLSchemaObject)
    value_tpe = value.type_str()
    checked = False
    convertted_value = None
    for expected_type in expected_types:
        convertted_value = (
            convert_collection_value(
                value,
                expected_type,
                filename,
                lineno,
                columno,
                vm=vm,
                config_meta=config_meta,
            )
            if not is_schema
            else value
        )
        checked, value_tpe = check_type(convertted_value, expected_type, vm=vm)
        if checked:
            break
    if not checked:
        if has_literal_type(expected_types):
            if isinstance(
                value,
                (
                    objpkg.KCLNoneObject,
                    objpkg.KCLTrueObject,
                    objpkg.KCLFalseObject,
                    objpkg.KCLIntObject,
                    objpkg.KCLFloatObject,
                ),
            ):
                value_tpe = f"{value_tpe}({value.value})"
            elif isinstance(value, objpkg.KCLStringObject):
                value_tpe = f'{value_tpe}("{value.value}")'
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.TypeError_Runtime_TYPE,
            file_msgs=[
                kcl_error.ErrFileMsg(filename=filename, line_no=lineno, col_no=columno)
            ],
            arg_msg="expect {}, got {}".format(
                common.get_tpes_str(expected_types).replace("@", ""),
                common.get_class_name(value_tpe),
            ),
        )
    return convertted_value


def has_literal_type(expected_types: List[str]) -> bool:
    for expected_type in expected_types:
        if is_literal_expected_type(expected_type):
            return True
        elif common.is_type_union(expected_type):
            for typ in common.split_type_union(expected_type):
                if is_literal_expected_type(typ):
                    return True
    return False


def is_none_or_undefined(value) -> bool:
    """Wether the value is None or Undefined"""
    return value is None or isinstance(
        value, (objpkg.UndefinedType, objpkg.KCLNoneObject, objpkg.KCLUndefinedObject)
    )


def check_type(value, expected_type: Optional[str], vm=None):
    value_tpe = value.type_str()
    # if expected type is a union type e.g. A|B|C int|str|[int]
    if not expected_type or expected_type == _KCL_TYPE_any:
        return True, value_tpe
    if is_none_or_undefined(value):
        return True, value_tpe
    if common.is_type_union(expected_type):
        return check_type_union(
            value,
            value_tpe,
            expected_type,
            vm=vm,
        )
    elif check_literal_type(value, expected_type):
        return True, expected_type
    elif check_number_multiplier_type(value, expected_type):
        return True, expected_type
    # if value type is a dict type e.g. {"k": "v"}
    elif isinstance(value, objpkg.KCLDictObject):
        return check_type_dict(
            value,
            expected_type,
            vm=vm,
        )
    # if value type is a list type e.g. [1, 2, 3]
    elif isinstance(value, objpkg.KCLListObject):
        return check_type_list(value, expected_type, vm=vm)
    elif value is not None and not isinstance(
        value, (objpkg.KCLNoneObject, objpkg.KCLUndefinedObject)
    ):
        # if value type is a built-in type e.g. str, int, float, bool
        if match_builtin_type(value_tpe, expected_type):
            return True, value_tpe
        # not list/dict, not built-in type, treat as user defined schema
        if isinstance(value, objpkg.KCLSchemaObject):
            return is_schema_expected_type(expected_type), value_tpe
        return False, value_tpe
    # Type Error
    return False, value_tpe


def is_schema_expected_type(expected_type: Optional[str]) -> bool:
    """Is scheam expected type"""
    if not expected_type:
        return True
    return (
        not common.islisttype(expected_type)
        and not common.isdicttype(expected_type)
        and not common.is_builtin_type(expected_type)
        and not is_literal_expected_type(expected_type)
    )


def is_literal_expected_type(expected_type: Optional[str]) -> bool:
    if expected_type in [_KCL_TYPE_None, _KCL_TYPE_True, _KCL_TYPE_False]:
        return True

    # str
    if expected_type.startswith('"'):
        return expected_type.endswith('"')
    if expected_type.startswith("'"):
        return expected_type.endswith("'")

    # int or float
    if expected_type.isdigit():
        return True
    if expected_type.replace(".", "", 1).isdigit() and expected_type.count(".") < 2:
        return True

    # non literal type
    return False


def check_literal_type(value, expected_type: Optional[str]):
    if not is_literal_expected_type(expected_type):
        return False
    # none
    if isinstance(value, objpkg.KCLNoneObject):
        return expected_type == _KCL_TYPE_None

    # bool
    if isinstance(value, objpkg.KCLFalseObject):
        return expected_type == _KCL_TYPE_False
    if isinstance(value, objpkg.KCLTrueObject):
        return expected_type == _KCL_TYPE_True

    # number
    if isinstance(value, (objpkg.KCLIntObject, objpkg.KCLFloatObject)):
        return f"{value.value}" == expected_type

    # str
    if isinstance(value, objpkg.KCLStringObject):
        return f"{value.value}" == literal_eval(expected_type)

    return False


def check_number_multiplier_type(value, expected_type: Optional[str]):
    """Check number multiplier"""
    if isinstance(value, objpkg.KCLNumberMultiplierObject):
        import kclvm.kcl.types.type_parser as type_parser

        if type_parser.is_number_multiplier_literal_type(expected_type):
            return str(value) == expected_type
        return expected_type == "units.NumberMultiplier"
    return False


def check_type_dict(
    value,
    expected_type,
    filename=None,
    lineno=None,
    columnno=None,
    vm=None,
):
    # Empty any type in [] or {:}
    if expected_type == "":
        return True, common.DICT_TYPE_NAME
    if not common.isdicttype(expected_type):
        return False, common.DICT_TYPE_NAME

    # validation None type on dict key and value
    expected_type = common.dereferencetype(expected_type)
    expected_key_type, expected_value_type = common.separate_kv(expected_type)
    if expected_key_type is None or expected_value_type is None:
        # either expected key type or value type can't be None
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.TypeError_Runtime_TYPE,
            file_msgs=[
                kcl_error.ErrFileMsg(filename=filename, line_no=lineno, col_no=columnno)
            ],
            arg_msg="error in dict type, key-value type can't be None",
        )
    expected_type = common.dereferencetype(expected_type)
    runtime_key_type, runtime_value_type = common.separate_kv(expected_type)
    if runtime_key_type is None or runtime_value_type is None:
        # either runtime key type or value type can't be None
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.TypeError_Runtime_TYPE,
            file_msgs=[
                kcl_error.ErrFileMsg(filename=filename, line_no=lineno, col_no=columnno)
            ],
            arg_msg="error in dict type, key-value type can't be None",
        )
    # foreach k,v in dict, check expected and runtime type of key and value
    for k, v in value.value.items():
        key_checked, value_checked = True, True
        key_value_tpe, value_value_tpe = "", ""
        # if no type is specified in dict, ignore drill-down type check
        if expected_value_type:
            value_checked, value_value_tpe = check_type(v, expected_value_type, vm=vm)
        if not key_checked or not value_checked:
            # shortcut on check failure
            return False, "{{{}:{}}}".format(key_value_tpe, value_value_tpe)
    return True, common.DICT_TYPE_NAME


def check_type_list(
    value,
    expected_type,
    vm=None,
):
    # Empty any type in [] or {:}
    if expected_type == "":
        return True, common.LIST_TYPE_NAME
    if not common.islisttype(expected_type):
        return False, common.LIST_TYPE_NAME
    expected_type = common.dereferencetype(expected_type)
    # foreach element in list, check expected and runtime type
    for i in value.value:
        checked, value_tpe = check_type(i, expected_type, vm=vm)
        if not checked:
            # shortcut on check failure
            return False, "[{}]".format(value_tpe)
    return True, common.LIST_TYPE_NAME


def check_type_builtin(
    value,
    expected_types,
    should_raise_err=True,
):
    if not expected_types:
        return True
    if any([tpe not in BUILTIN_TYPES for tpe in expected_types]):
        return True
    if any([match_builtin_type(value.type_str(), tpe) for tpe in expected_types]):
        return True
    if should_raise_err:
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.TypeError_Runtime_TYPE,
            arg_msg="expect {}, got {}".format(
                common.get_tpes_str(expected_types).replace("@", ""),
                common.get_class_name(value.type_str()),
            ),
        )
    return False


def match_builtin_type(value_tpe, expected_type):
    return (
        value_tpe == expected_type
        or value_tpe == common.CLASS_TYPE_TMPL.format(expected_type)
        or (value_tpe == INT_TYPE and expected_type == FLOAT_TYPE)
    )


def check_type_union(value, value_tpe, expected_type, vm=None):
    """
    Match built-in union type or single built-in type
    """
    expected_types = common.split_type_union(expected_type)
    if len(expected_types) == 1:
        return False
    return (
        any(
            [
                check_type(
                    value,
                    tpe,
                    vm=vm,
                )[0]
                for tpe in expected_types
            ]
        ),
        value_tpe,
    )
