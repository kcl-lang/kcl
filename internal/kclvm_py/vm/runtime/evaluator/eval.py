from dataclasses import dataclass
from typing import List, Tuple, Union, cast
from copy import deepcopy

import kclvm.kcl.error as kcl
import kclvm.kcl.ast as ast
import kclvm.api.object as objpkg
from kclvm.vm.code import Opcode
from kclvm.compiler.check.check_type import type_pack_and_check
from kclvm.compiler.extension.builtin.system_module import json as _json
from kclvm.compiler.extension.builtin.system_module import yaml as _yaml

from .union import union, resolve_schema_obj
from .common import plus, handle_subscript


BINARY_FUNCTIONS = {
    Opcode.BINARY_ADD: lambda x, y: plus(x, y),
    Opcode.BINARY_SUBTRACT: lambda x, y: x - y,
    Opcode.BINARY_MULTIPLY: lambda x, y: x * y,
    Opcode.BINARY_TRUE_DIVIDE: lambda x, y: x / y,
    Opcode.BINARY_FLOOR_DIVIDE: lambda x, y: x // y,
    Opcode.BINARY_LSHIFT: lambda x, y: x << y,
    Opcode.BINARY_RSHIFT: lambda x, y: x >> y,
    Opcode.BINARY_POWER: lambda x, y: x ** y,
    Opcode.BINARY_SUBSCR: lambda x, y: handle_subscript(x, y),
    Opcode.BINARY_XOR: lambda x, y: x ^ y,
    Opcode.BINARY_AND: lambda x, y: x & y,
    Opcode.BINARY_OR: lambda x, y: union(x, y),
    Opcode.BINARY_MODULO: lambda x, y: x % y,
    Opcode.BINARY_LOGIC_AND: lambda x, y: x and y,
    Opcode.BINARY_LOGIC_OR: lambda x, y: x or y,
    Opcode.COMPARE_EQUAL_TO: lambda x, y: x == y,
    Opcode.COMPARE_NOT_EQUAL_TO: lambda x, y: x != y,
    Opcode.COMPARE_LESS_THAN: lambda x, y: x < y,
    Opcode.COMPARE_LESS_THAN_OR_EQUAL_TO: lambda x, y: x <= y,
    Opcode.COMPARE_GREATER_THAN: lambda x, y: x > y,
    Opcode.COMPARE_GREATER_THAN_OR_EQUAL_TO: lambda x, y: x >= y,
    Opcode.COMPARE_IS: lambda x, y: x is y,
    Opcode.COMPARE_IS_NOT: lambda x, y: x is not y,
    Opcode.COMPARE_IN: lambda x, y: x in y,
    Opcode.COMPARE_NOT_IN: lambda x, y: x not in y,
}

UNARY_FUNCTIONS = {
    Opcode.UNARY_INVERT: lambda x: ~x,
    Opcode.UNARY_NOT: lambda x: not x,
    Opcode.UNARY_POSITIVE: lambda x: +x,
    Opcode.UNARY_NEGATIVE: lambda x: -x,
}

INPLACE_FUNCTIONS = {
    Opcode.INPLACE_ADD: lambda x, y: plus(x, y),
    Opcode.INPLACE_SUBTRACT: lambda x, y: x - y,
    Opcode.INPLACE_MULTIPLY: lambda x, y: x * y,
    Opcode.INPLACE_TRUE_DIVIDE: lambda x, y: x / y,
    Opcode.INPLACE_FLOOR_DIVIDE: lambda x, y: x // y,
    Opcode.INPLACE_LSHIFT: lambda x, y: x << y,
    Opcode.INPLACE_RSHIFT: lambda x, y: x >> y,
    Opcode.INPLACE_POWER: lambda x, y: x ** y,
    Opcode.INPLACE_XOR: lambda x, y: x ^ y,
    Opcode.INPLACE_AND: lambda x, y: x & y,
    Opcode.INPLACE_OR: lambda x, y: union(x, y),
    Opcode.INPLACE_MODULO: lambda x, y: x % y,
}


@dataclass
class Evaluator:
    """Evaluator is a class responsible for parsing
    KCL objects and performing interpretation to get results
    """

    # Binary

    def eval_binary_op(
        self,
        left: objpkg.KCLObject,
        right: objpkg.KCLObject,
        code: Union[int, Opcode],
        vm=None,
    ) -> objpkg.KCLObject:
        if not left or not right or not code:
            raise Exception(f"invalid binary opcode action {left}, {right} and {code}")
        func = BINARY_FUNCTIONS.get(code)
        if not func:
            raise Exception(f"invalid binary opcode {code}")
        if code == Opcode.BINARY_OR:
            result = union(
                deepcopy(left),
                right,
                or_mode=True,
                should_config_resolve=True,
                should_idempotent_check=True,
                vm=vm,
            )
        elif code == Opcode.BINARY_ADD or code == Opcode.BINARY_SUBSCR:
            result = func(left, right)
        else:
            result = objpkg.to_kcl_obj(
                func(objpkg.to_python_obj(left), objpkg.to_python_obj(right))
            )
        return result

    # Unary

    def eval_unary_op(
        self, obj: objpkg.KCLObject, code: Union[int, Opcode]
    ) -> objpkg.KCLObject:
        if not obj or not code:
            raise Exception(f"invalid binary opcode action {obj} and {code}")
        func = UNARY_FUNCTIONS.get(code)
        if not func:
            raise Exception(f"invalid unary opcode {code}")
        r = objpkg.to_kcl_obj(func(obj.value))
        return r

    # Inplace operator

    def eval_inplace_op(
        self,
        left: objpkg.KCLObject,
        right: objpkg.KCLObject,
        code: Union[int, Opcode],
        vm=None,
    ) -> objpkg.KCLObject:
        if not left or not right or not code:
            raise Exception(f"invalid inpalce opcode action {left}, {right} and {code}")
        func = INPLACE_FUNCTIONS.get(code)
        if not func:
            raise Exception(f"invalid inplace opcode {code}")
        if code == Opcode.INPLACE_OR:
            result = union(
                left,
                right,
                or_mode=True,
                should_config_resolve=True,
                should_idempotent_check=True,
                vm=vm,
            )
        elif code == Opcode.INPLACE_ADD:
            result = func(left, right)
        else:
            result = objpkg.to_kcl_obj(
                func(
                    objpkg.to_python_obj(left.value), objpkg.to_python_obj(right.value)
                )
            )
        return result

    # Compare

    def eval_compare_op(
        self,
        left: objpkg.KCLObject,
        right: objpkg.KCLObject,
        code: Union[int, Opcode],
        vm=None,
    ) -> objpkg.KCLObject:
        # Avoid the overhead of boxing and unboxing large objects.
        if isinstance(right, (objpkg.KCLDictObject, objpkg.KCLSchemaObject)):
            if code == Opcode.COMPARE_IN:
                return objpkg.to_kcl_obj(left in right)
            elif code == Opcode.COMPARE_NOT_IN:
                return objpkg.to_kcl_obj(left not in right)
            else:
                return self.eval_binary_op(left, right, code, vm=vm)
        else:
            return self.eval_binary_op(left, right, code, vm=vm)

    # Iterable

    def iter_next(self, obj: objpkg.KCLObject) -> objpkg.KCLObject:
        if obj.type() != objpkg.KCLObjectType.ITER:
            kcl.report_exception(
                err_type=kcl.ErrType.EvaluationError_TYPE,
                arg_msg="only iterable object has next function",
            )
        return objpkg.to_kcl_obj(obj.next())

    # Functions

    def call_vars_and_keywords(
        self, argc: int, vm
    ) -> Tuple[List[objpkg.KCLObject], List[objpkg.KWArg]]:
        n_args = int(argc & 0xFF)
        n_kwargs = int((argc >> 8) & 0xFF)
        p, q = len(vm.stack) - 2 * n_kwargs, len(vm.stack)
        args = []
        kwargs = []
        for i in range((q - p) // 2 - 1, -1, -1):
            v = vm.pop()
            kstr = vm.pop()
            kwargs.append(objpkg.KWArg(name=kstr, value=v))
        p, q = p - n_args, p
        for i in range(q - p - 1, -1, -1):
            arg = vm.pop()
            args.append(arg)
        return args[::-1], kwargs[::-1]

    def eval_call(self, code: Union[int, Opcode], argc: int, vm) -> objpkg.KCLObject:
        if code == Opcode.CALL_FUNCTION:
            # Function callable without `*args` and `**kwargs`
            args, kwargs = self.call_vars_and_keywords(argc, vm)
            callable_obj = vm.pop()
            if isinstance(callable_obj, objpkg.KCLCompiledFunctionObject):
                vm.push_frame_using_callable(
                    callable_obj.pkgpath,
                    callable_obj,
                    (args if args else []),
                    kwargs,
                    args_len=len(args),
                )
                return objpkg.NONE_INSTANCE
            elif isinstance(callable_obj, objpkg.KCLFunctionObject):
                result_obj = callable_obj.call(args, kwargs, vm)
                vm.push(result_obj)
                return result_obj
            elif isinstance(callable_obj, objpkg.KCLSchemaTypeObject):
                schema_type_obj = cast(objpkg.KCLSchemaTypeObject, callable_obj)
                inst = schema_type_obj.new_instance({}, {}, args, kwargs, vm)
                vm.push(inst)
                return inst
            elif isinstance(
                callable_obj, (objpkg.KCLNoneObject, objpkg.KCLUndefinedObject)
            ):
                # Ignore the user None callable
                vm.push(callable_obj)
                return callable_obj
            else:
                kcl.report_exception(
                    err_type=kcl.ErrType.EvaluationError_TYPE,
                    arg_msg=f"'{callable_obj.type()}' object is not callable",
                )

    # Attribute and subscript

    def load_attr(self, obj: objpkg.KCLObject, attr: str) -> objpkg.KCLObject:
        """Get attribute value of a KCL object"""
        if obj.type() in [
            objpkg.KCLObjectType.DICT,
            objpkg.KCLObjectType.SCHEMA,
            objpkg.KCLObjectType.MODULE,
        ]:
            return obj.get(attr)
        elif obj.type() in [
            objpkg.KCLObjectType.STRING,
            objpkg.KCLObjectType.SCHEMA_TYPE,
        ]:
            return obj.get_member_method(attr)
        else:
            kcl.report_exception(
                err_type=kcl.ErrType.EvaluationError_TYPE,
                arg_msg=f"'{obj.type_str()}' object has no attribute '{attr}'",
            )

    def set_attr(
        self,
        obj: objpkg.KCLObject,
        item: objpkg.KCLObject,
        value: objpkg.KCLObject,
        vm=None,
    ) -> objpkg.KCLObject:
        """Set attribute value of KCL object"""
        if not isinstance(obj, (objpkg.KCLDictObject, objpkg.KCLSchemaObject)):
            kcl.report_exception(
                err_type=kcl.ErrType.EvaluationError_TYPE,
                arg_msg="only schema and dict object can be updated attribute",
            )
        obj.update_key_value(item, value)
        if isinstance(obj, objpkg.KCLSchemaObject):
            obj = resolve_schema_obj(obj, obj.config_keys | {item.value}, vm=vm)
        return obj

    def set_item(
        self, obj: objpkg.KCLObject, item: objpkg.KCLObject, value: objpkg.KCLObject
    ) -> objpkg.KCLObject:
        """Set subscript value of a KCL object"""
        obj.value[item.value] = value
        return obj

    # List

    def list_append(
        self, obj: objpkg.KCLObject, item: objpkg.KCLObject
    ) -> objpkg.KCLObject:
        """Append an item into list"""
        if obj.type() != objpkg.KCLObjectType.LIST:
            kcl.report_exception(
                err_type=kcl.ErrType.EvaluationError_TYPE,
                arg_msg="only list object can append value",
            )
        obj.append(item)
        return obj

    # Dict

    def dict_append(
        self, obj: objpkg.KCLObject, key: objpkg.KCLObject, value: objpkg.KCLObject
    ) -> objpkg.KCLObject:
        """Append an key-value pair into dict"""
        if obj.type() != objpkg.KCLObjectType.DICT:
            if obj.type() != objpkg.KCLObjectType.LIST:
                kcl.report_exception(
                    err_type=kcl.ErrType.EvaluationError_TYPE,
                    arg_msg="only dict object can append key-value pair",
                )
        obj.update_key_value(key, value)
        return obj

    # String Format value

    def format_value(
        self, obj: objpkg.KCLObject, format_spec: str
    ) -> objpkg.KCLStringObject:
        if not format_spec:
            value = "{}".format(objpkg.to_python_obj(obj))
            return objpkg.KCLStringObject(value=value)
        assert isinstance(format_spec, str)
        if format_spec.lower() == "#json":
            value = _json.KMANGLED_encode(objpkg.to_python_obj(obj))
            return objpkg.KCLStringObject(value=value)
        if format_spec.lower() == "#yaml":
            value = _yaml.KMANGLED_encode(objpkg.to_python_obj(obj))
            return objpkg.KCLStringObject(value=value)
        kcl.report_exception(
            err_type=kcl.ErrType.InvalidFormatSpec_TYPE,
            arg_msg=kcl.INVALID_FORMAT_SPEC_MSG.format(format_spec),
        )

    # Schema config operation expression

    def update_schema_attr(
        self,
        attr: str,
        schema_obj: objpkg.KCLSchemaObject,
        config_value: objpkg.KCLObject,
        conf_meta: dict,
        expected_types: List[str],
        operation=ast.ConfigEntryOperation.UNION,
        index=None,
        vm=None,
        filename=None,
        lineno=None,
        columnno=None,
    ):
        """Update schema attr value with config dict"""
        if index is None or index < 0:
            config_value_checked = type_pack_and_check(
                config_value,
                expected_types,
                vm=vm,
                filename=filename,
                lineno=lineno,
                columno=columnno,
                config_meta=conf_meta.get("$conf_meta"),
            )
        else:
            value = schema_obj.get(attr)
            if not isinstance(value, objpkg.KCLListObject):
                kcl.report_exception(
                    err_type=kcl.ErrType.EvaluationError_TYPE,
                    arg_msg="only list attribute can be inserted value",
                )
            config_value_checked = type_pack_and_check(
                objpkg.KCLListObject(items=[config_value]),
                expected_types,
                vm=vm,
                filename=filename,
                lineno=lineno,
                columno=columnno,
                config_meta=conf_meta.get("$conf_meta"),
            )
            config_value_checked = cast(
                objpkg.KCLListObject, config_value_checked
            ).items[0]
        SCHEMA_CONFIG_OP_MAPPING = {
            ast.ConfigEntryOperation.UNION: self.union_schema_attr,
            ast.ConfigEntryOperation.OVERRIDE: self.override_schema_attr,
            ast.ConfigEntryOperation.INSERT: self.insert_schema_attr,
        }
        func = SCHEMA_CONFIG_OP_MAPPING.get(operation)
        if not func:
            raise Exception(f"Invalid schema config object operation: {operation}")
        func(attr, schema_obj, config_value_checked, index, vm)

    def union_schema_attr(
        self,
        attr: str,
        schema_obj: objpkg.KCLSchemaObject,
        config_value: objpkg.KCLObject,
        index=None,
        vm=None,
    ):
        if index is None or index < 0:
            # TODO: modify `should_idempotent_check` to False after Konfig code update finish
            schema_obj.union_with({attr: config_value}, should_idempotent_check=False)
        else:
            # Union with list internal index value
            value = schema_obj.get(attr)
            value.items[index] = union(
                value.items[index], config_value, should_idempotent_check=False, vm=vm
            )
            schema_obj.union_with({attr: value}, should_idempotent_check=False)

    def override_schema_attr(
        self,
        attr: str,
        schema_obj: objpkg.KCLSchemaObject,
        config_value: objpkg.KCLObject,
        index=None,
        vm=None,
    ):
        if index is None or index < 0:
            schema_obj.update_key_value(attr, config_value)
        else:
            schema_obj.list_key_override(attr, config_value, index)

    def insert_schema_attr(
        self,
        attr: str,
        schema_obj: objpkg.KCLSchemaObject,
        config_value: objpkg.KCLObject,
        index=None,
        vm=None,
    ):
        if index is not None and index >= 0:
            config_value = objpkg.KCLListObject(items=[config_value])
        schema_obj.insert_with(attr, config_value, index)
