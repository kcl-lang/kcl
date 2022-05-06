# Copyright 2021 The KCL Authors. All rights reserved.

import time
from typing import Optional
from copy import deepcopy

import kclvm.api.object as obj
import kclvm.kcl.error as kcl
import kclvm.kcl.ast as ast
import kclvm.kcl.types as types
import kclvm.config
import kclvm.compiler.extension.builtin

from kclvm.vm.runtime.evaluator.eval import Evaluator
from kclvm.vm.code import Opcode
from kclvm.compiler.check.check_type import (
    type_pack_and_check,
    runtime_types,
)
from kclvm.unification import value_subsume

_evaluator_inst = Evaluator()


def is_kcl_debug():
    return kclvm.config.debug and kclvm.config.verbose > 2


def debug_stack(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    idx = int(arg & 0xFF)
    at = int((arg >> 8) & 0xFF)
    return vm.debug_stack(idx, at)


def debug_locals(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    idx = int(arg & 0xFF)
    at = int((arg >> 8) & 0xFF)
    return vm.debug_locals(idx, at)


def debug_globals(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    idx = int(arg & 0xFF)
    at = int((arg >> 8) & 0xFF)
    return vm.debug_globals(idx, at)


def debug_names(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    idx = int(arg & 0xFF)
    at = int((arg >> 8) & 0xFF)
    return vm.debug_names(idx, at)


def unpack_operand(arg: int):
    return int(arg & 0xFF), int((arg >> 8) & 0xFF), int((arg >> 16) & 0xFF)


def bin_action(vm, code: int, _arg: int) -> Optional[obj.KCLObject]:
    right, left = vm.pop(), vm.top()
    result = _evaluator_inst.eval_binary_op(left, right, code, vm=vm)
    vm.set_top(result)
    return result


def unary_action(vm, code: int, _arg: int) -> Optional[obj.KCLObject]:
    expr_obj = vm.top()
    result = _evaluator_inst.eval_unary_op(expr_obj, code)
    vm.set_top(result)
    return result


def inplace_action(vm, code: int, _arg: int) -> Optional[obj.KCLObject]:
    value, target = vm.pop(), vm.top()
    result = _evaluator_inst.eval_inplace_op(target, value, code, vm=vm)
    vm.set_top(result)
    return result


def bin_add(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return bin_action(vm, code, arg)


def bin_substract(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return bin_action(vm, code, arg)


def bin_floor_divide(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return bin_action(vm, code, arg)


def bin_true_divide(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return bin_action(vm, code, arg)


def build_map(vm, _code: int, _arg: int) -> Optional[obj.KCLObject]:
    dict_obj = obj.KCLDictObject(value={})
    vm.push(dict_obj)
    return dict_obj


def load_free(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    free_obj = obj.NONE_INSTANCE
    index = vm.frame_index
    name = vm.names[arg]
    # Search the local variables from the inside to the outside schema
    while index >= 1:
        index -= 1
        if name in vm.frames[index].locals:
            free_obj = vm.frames[index].locals[name]
    vm.push(free_obj)
    return free_obj


def load_closure(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    # free_obj = vm.ctx.free_vars[arg]
    closure_obj = (
        vm.ctx.free_vars[arg] if arg < len(vm.ctx.free_vars) else obj.UNDEFINED_INSTANCE
    )
    vm.push(closure_obj)
    return closure_obj


def make_function(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return make_compiled_function(vm, code, arg)


def make_closure(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return make_compiled_function(vm, code, arg, is_closure=True)


def make_compiled_function(
    vm, _code: int, arg: int, is_closure: bool = False
) -> Optional[obj.KCLObject]:
    """
    Pushes a new function object on the stack. From bottom to top, the consumed stack must consist of values if the argument carries a specified flag value
    + 0x01 a tuple of default values for positional-only and positional-or-keyword parameters in positional order
    + 0x02 a dictionary of keyword-only parameters’ default values
    + 0x04 an annotation dictionary
    + 0x08 a tuple containing cells for free variables, making a closure
    + the code associated with the function (at TOS1)
    + the qualified name of the function (at TOS)
    """
    # Pop the function name
    name = vm.pop().value
    # Pop code placeholder
    runtime_code = vm.pop()
    closure = vm.pop() if is_closure else None
    args, _, _ = unpack_operand(arg)
    # var params []grumpy.Param
    params: list = [0] * args
    index = 0
    for index in range(args):
        # positional args placeholders to fit grumpy arg validate
        arg_value = vm.pop()
        arg_type = vm.pop().value
        arg_name = vm.pop().value
        params[index] = obj.Parameter(
            name=arg_name, type_annotation=arg_type, value=arg_value
        )

    func = obj.KCLCompiledFunctionObject(
        name=name,
        instructions=runtime_code.codes,
        names=runtime_code.names,
        constants=runtime_code.constants,
        num_locals=0,
        num_parameters=arg,
        params=params[::-1],
        pkgpath=vm.cur_run_pkg,
        closure=closure,
    )
    vm.push(func)
    return func


def pop_top(vm, _code: int, _arg: int) -> Optional[obj.KCLObject]:
    return vm.pop()


def rot_two(vm, _code: int, _arg: int) -> Optional[obj.KCLObject]:
    v = vm.pop()
    w = vm.pop()
    vm.push(v)
    vm.push(w)
    return w


def rot_three(vm, _code: int, _arg: int) -> Optional[obj.KCLObject]:
    v = vm.pop()
    w = vm.pop()
    u = vm.pop()
    vm.push(v)
    vm.push(w)
    vm.push(u)
    return u


def dup_top(vm, _code: int, _arg: int) -> Optional[obj.KCLObject]:
    vm.push(vm.peek())
    return vm.peek()


def copy_top(vm, _code: int, _arg: int) -> Optional[obj.KCLObject]:
    vm.push(deepcopy(vm.peek()))
    return vm.peek()


def dup_top_two(vm, _code: int, _arg: int) -> Optional[obj.KCLObject]:
    v = vm.pop()
    w = vm.pop()
    vm.push(w)
    vm.push(v)
    vm.push(w)
    vm.push(v)
    return v


def nop(_vm, _code: int, _arg: int) -> Optional[obj.KCLObject]:
    """Empty opcode action"""
    pass


def unary_pos(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return unary_action(vm, code, arg)


def unary_neg(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return unary_action(vm, code, arg)


def unary_not(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return unary_action(vm, code, arg)


def unary_invert(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return unary_action(vm, code, arg)


def bin_power(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return bin_action(vm, code, arg)


def bin_mul(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return bin_action(vm, code, arg)


def bin_mod(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return bin_action(vm, code, arg)


def bin_subscr(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return bin_action(vm, code, arg)


def inplace_floor_div(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return inplace_action(vm, code, arg)


def inplace_true_div(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return inplace_action(vm, code, arg)


def store_map(vm, _code: int, _arg: int) -> Optional[obj.KCLObject]:
    v, k = vm.pop(), vm.pop()
    config_ref = vm.peek()
    if v.type() == obj.KCLObjectType.UNPACK and k.type() in [
        obj.KCLObjectType.NONE,
        obj.KCLObjectType.UNDEFINED,
    ]:
        config_ref.append_unpack(v.unpack())
    else:
        config_ref.update_key_value(k, v)
    return config_ref


def inplace_add(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return inplace_action(vm, code, arg)


def inplace_sub(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return inplace_action(vm, code, arg)


def inplace_mul(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return inplace_action(vm, code, arg)


def inplace_mod(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return inplace_action(vm, code, arg)


def store_subscr(vm, _code: int, _arg: int) -> Optional[obj.KCLObject]:
    """v[w] = u"""
    w = vm.pop()
    v = vm.pop()
    u = vm.pop()
    _evaluator_inst.set_item(v, w, u)
    return v


def bin_lshift(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return bin_action(vm, code, arg)


def bin_rshift(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return bin_action(vm, code, arg)


def bin_and(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return bin_action(vm, code, arg)


def bin_xor(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return bin_action(vm, code, arg)


def bin_or(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return bin_action(vm, code, arg)


def bin_logic_and(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return bin_action(vm, code, arg)


def bin_logic_or(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return bin_action(vm, code, arg)


def inplace_pow(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return inplace_action(vm, code, arg)


def get_iter(vm, _code: int, _arg: int) -> Optional[obj.KCLObject]:
    assert 0 < _arg <= 2
    iter_obj = obj.KCLIterObject.build_iter(vm.top(), _arg)
    vm.set_top(iter_obj)
    return None


def print_expr(vm, _code: int, _arg: int) -> Optional[obj.KCLObject]:
    expr_obj = vm.pop()
    print(expr_obj.value)
    return expr_obj


def emit_expr(vm, _code: int, _arg: int) -> Optional[obj.KCLObject]:
    emitted_obj = vm.pop()
    if isinstance(emitted_obj, obj.KCLSchemaObject):
        output_type = (
            emitted_obj.get(obj.SCHEMA_SETTINGS_ATTR_NAME)
            .get(obj.SETTINGS_OUTPUT_KEY)
            .value
        )
        if (
            output_type == obj.SETTINGS_OUTPUT_STANDALONE
            or output_type == obj.SETTINGS_OUTPUT_IGNORE
        ):
            # Internal magic variable
            vm.update_global(f"$EMIT_VAR_{time.time()}", emitted_obj)
    return emitted_obj


def inplace_lshift(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return inplace_action(vm, code, arg)


def inplace_rlshift(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return inplace_action(vm, code, arg)


def inplace_and(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return inplace_action(vm, code, arg)


def inplace_xor(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return inplace_action(vm, code, arg)


def inplace_or(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    return inplace_action(vm, code, arg)


def return_value(vm, _code: int, _arg: int) -> Optional[obj.KCLObject]:
    vm.pop_frame()
    return vm.peek()


def return_last_value(vm, _code: int, _arg: int) -> Optional[obj.KCLObject]:
    frame = vm.pop_frame()
    variables = list(frame.locals.values())
    return_obj = variables[-1] if variables else obj.NONE_INSTANCE
    vm.push(return_obj)
    return return_obj


def store_name(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    return vm.store_name(arg)


def unpack_sequence(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    sequence_obj = vm.pop()
    unpack_obj = obj.KCLUnpackObject(sequence_obj, arg != 1)
    vm.push(unpack_obj)
    return unpack_obj


def for_iter(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    """
    TOS is an iterator. Call its iter_next() method. If this yields a new value,
    push it on the stack (leaving the iterator below it).
    If the iterator indicates it is exhausted TOS is popped,
    and the byte code counter is incremented by 'arg' argument.
    """
    try:
        iter_next_obj = _evaluator_inst.iter_next(vm.peek())
        assert isinstance(iter_next_obj, obj.KCLTupleObject)
        # Push loop variables
        for o in iter_next_obj.value[::-1]:
            vm.push(obj.to_kcl_obj(o))
    except StopIteration:
        it = vm.pop()
        assert isinstance(it, obj.KCLIterObject)
        vm.set_instruction_pointer(arg)
    except Exception as err:
        kcl.report_exception(
            err_type=kcl.ErrType.EvaluationError_TYPE, arg_msg=str(err)
        )

    return None


def store_attr(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    """v.w = u"""
    w = vm.names[arg]
    v = vm.pop()
    u = vm.pop()

    _evaluator_inst.set_attr(v, obj.KCLStringObject(w), u, vm=vm)

    return v


def store_global(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    return vm.store_name(arg)


def load_const(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    return vm.load_const(arg)


def load_name(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    return vm.load_name(arg)


def build_list(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    items = [vm.pop() for _i in range(arg)][::-1]
    list_obj = obj.KCLListObject(items=[])
    for item in items:
        if item.type() == obj.KCLObjectType.UNPACK:
            list_obj.append_unpack(item.unpack())
        else:
            list_obj.append(item)
    vm.push(list_obj)
    return list_obj


def schema_attr(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    """
    Define a schema attr into schema type object
    """
    n_decorators = arg
    decorators_obj = [vm.pop() for _i in range(n_decorators)][::-1]
    expected_types = [vm.pop().value]
    attr, value, has_default, is_final, is_optional, op_code = (
        vm.pop().value,
        vm.pop(),
        vm.pop().value,
        vm.pop().value,
        vm.pop().value,
        vm.pop().value,
    )
    config_meta = vm.lazy_eval_ctx.config_meta
    config_obj = vm.ctx.locals[obj.SCHEMA_CONFIG_VALUE_KEY]
    schema_obj = vm.ctx.locals[obj.SCHEMA_SELF_VALUE_KEY]
    config_value = config_obj.get(attr)
    schema_obj.set_attr_type(attr, expected_types)
    schema_obj.set_attr_optional(attr, is_optional)
    attr_runtime_types = runtime_types(expected_types, vm=vm)
    if attr_runtime_types:
        schema_obj.set_attr_runtime_type(attr, attr_runtime_types)
    for decorator in decorators_obj:
        schema_obj.add_decorator(attr, decorator)

    operation = (
        config_obj.get_operation(attr)
        if isinstance(config_obj, obj.KCLSchemaConfigObject)
        else ast.ConfigEntryOperation.UNION
    )
    insert_index = (
        config_obj.get_insert_index(attr)
        if isinstance(config_obj, obj.KCLSchemaConfigObject)
        else None
    )

    # Update and union with config_value
    if has_default:
        target = (
            schema_obj.get(attr) if attr in schema_obj else obj.KCLNoneObject.instance()
        )
        if op_code is None:
            op_value = value
        else:
            op_value = _evaluator_inst.eval_inplace_op(target, value, op_code, vm=vm)
        op_value = type_pack_and_check(op_value, expected_types, vm=vm)
        if schema_obj.get_immutable_flag(attr) and target.value != op_value.value:
            kcl.report_exception(
                err_type=kcl.ErrType.ImmutableRuntimeError_TYPE,
                arg_msg=f"final schema field '{attr}'",
            )
        vm.lazy_eval_ctx.set_value(attr, op_value)
    elif attr not in schema_obj:
        schema_obj.update({attr: obj.KCLUndefinedObject.instance()})

    if attr in config_obj:
        conf_meta = config_meta.get(attr, {})
        filename = conf_meta.get("filename")
        lineno = conf_meta.get("lineno")
        columnno = conf_meta.get("columnno")
        if is_final and obj.to_python_obj(config_value) != obj.to_python_obj(
            schema_obj.attrs.get(attr)
        ):
            kcl.report_exception(
                err_type=kcl.ErrType.ImmutableRuntimeError_TYPE,
                file_msgs=[
                    kcl.ErrFileMsg(filename=filename, line_no=lineno, col_no=columnno)
                ],
                arg_msg=f"final schema field '{attr}'",
            )
        _evaluator_inst.update_schema_attr(
            attr=attr,
            schema_obj=schema_obj,
            config_value=config_value,
            conf_meta=conf_meta,
            expected_types=expected_types,
            operation=operation,
            index=insert_index,
            vm=vm,
            filename=filename,
            lineno=lineno,
            columnno=columnno,
        )
        vm.lazy_eval_ctx.set_value(attr, schema_obj.get(attr))

    # Update mutable flag
    schema_obj.set_immutable_flag(attr, is_final)
    vm.update_local(attr, schema_obj.get(attr))
    return schema_obj


def schema_update_attr(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    name = vm.pop().value
    value = vm.pop()

    schema_obj = vm.ctx.locals[obj.SCHEMA_SELF_VALUE_KEY]
    config_obj = vm.ctx.locals[obj.SCHEMA_CONFIG_VALUE_KEY]
    config_meta = vm.lazy_eval_ctx.config_meta
    config_value = config_obj.get(name)

    # A private schema attribute can be added into schema object whether it's relaxed or not
    if schema_obj.get_immutable_flag(name):
        kcl.report_exception(
            err_type=kcl.ErrType.ImmutableRuntimeError_TYPE,
            arg_msg=f"final schema field '{name}'",
        )

    conf_meta = config_meta.get(name, {})
    filename = conf_meta.get("filename")
    lineno = conf_meta.get("lineno")
    columnno = conf_meta.get("columnno")
    if name not in schema_obj or not schema_obj.get_attr_type(name):
        schema_obj.set_attr_optional(name, True)
    vm.lazy_eval_ctx.set_value(
        name, type_pack_and_check(value, schema_obj.get_attr_type(name), vm=vm)
    )

    if name in config_obj:
        config_value = type_pack_and_check(
            config_value,
            schema_obj.get_attr_type(name),
            vm=vm,
            filename=filename,
            lineno=lineno,
            columno=columnno,
            config_meta=conf_meta.get("$conf_meta"),
        )
        cfg_obj = obj.KCLSchemaConfigObject(value={name: config_value})
        cfg_obj.update_attr_op_using_obj(config_obj)
        schema_obj.union_with(cfg_obj)
        vm.lazy_eval_ctx.set_value(name, schema_obj.get(name))


def make_decorator(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    decorator = vm.pop()
    assert isinstance(decorator, obj.KCLDecoratorObject)
    args, kwargs = _evaluator_inst.call_vars_and_keywords(arg, vm)
    decorator.resolve(args, kwargs)
    vm.push(decorator)
    return decorator


def schema_load_attr(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    name_obj = vm.top()
    vm.set_top(vm.lazy_eval_ctx.get_value(name_obj.value))
    return name_obj


def import_name(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    return vm.import_name(_code, arg)


def jump_forward(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    vm.set_instruction_pointer(arg)
    return None


def jump_if_false_or_pop(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    condition_obj = vm.peek()
    if not obj.KCLObject.is_truthy(condition_obj):
        vm.set_instruction_pointer(arg)
    else:
        vm.pop()
    return None


def jump_if_true_or_pop(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    if obj.KCLObject.is_truthy(vm.peek()):
        vm.set_instruction_pointer(arg)
    else:
        vm.pop()
    return None


def jump_absolute(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    vm.set_instruction_pointer(arg)
    return None


def pop_jump_if_false(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    condition_obj = vm.pop()
    if not obj.KCLObject.is_truthy(condition_obj):
        vm.set_instruction_pointer(arg)
    return None


def pop_jump_if_true(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    condition_obj = vm.pop()
    if obj.KCLObject.is_truthy(condition_obj):
        vm.set_instruction_pointer(arg)
    return None


def load_global(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    vm.load_name(arg)
    return None


def raise_varargs(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    if arg != 1:
        raise Exception(f"invalid raise_varargs opcode arg {arg}")
    msg_obj = vm.pop()
    msg = msg_obj.value if msg_obj.type() == obj.KCLObjectType.STRING else ""
    if arg == 1:
        kcl.report_exception(
            err_type=kcl.ErrType.AssertionError_TYPE, arg_msg=msg if msg else ""
        )
    return None


def raise_check(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    if arg != 1:
        raise Exception(f"invalid raise_check opcode arg {arg}")
    msg_obj = vm.pop()
    msg = msg_obj.value if msg_obj.type() == obj.KCLObjectType.STRING else ""
    if arg == 1:
        config_meta = obj.to_python_obj(vm.ctx.locals[obj.SCHEMA_CONFIG_META_KEY])
        conf_filename, conf_line, conf_column = (
            config_meta.get("$filename"),
            config_meta.get("$lineno"),
            config_meta.get("$columnno"),
        )
        filename, line, _ = vm.get_info(True)
        kcl.report_exception(
            err_type=kcl.ErrType.SchemaCheckFailure_TYPE,
            file_msgs=[
                kcl.ErrFileMsg(
                    filename=filename,
                    line_no=line,
                    arg_msg=kcl.SCHEMA_CHECK_FILE_MSG_COND,
                ),
                kcl.ErrFileMsg(
                    filename=conf_filename,
                    line_no=conf_line,
                    col_no=conf_column,
                    arg_msg=kcl.SCHEMA_CHECK_FILE_MSG_ERR,
                ),
            ],
            arg_msg=msg if msg else "",
        )
    return None


def load_local(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    vm.load_local(arg)
    return None


def store_local(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    vm.store_local(arg)
    return None


def call_function(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    _evaluator_inst.eval_call(code, arg, vm)
    return None


def build_slice(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    if arg != 2 and arg != 3:
        raise Exception(f"invalid slice argc {arg}")
    step = obj.KCLNoneObject.instance()
    if arg == 3:
        step = vm.pop()
    stop = vm.pop()
    start = vm.top()
    vm.set_top(obj.KCLSliceObject(start=start, stop=stop, step=step))
    return None


def list_append(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    item = vm.pop()
    list_obj = vm.peek_nth(arg)
    _evaluator_inst.list_append(list_obj, item)
    return list_obj


def map_add(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    operation = vm.pop().value
    key = vm.pop()
    value = vm.pop()
    config_ref = vm.peek_nth(arg)
    data_obj = obj.KCLSchemaConfigObject(value={key.value: value})
    if operation is None or operation == ast.ConfigEntryOperation.UNION:
        config_ref.union_with(data_obj)
    elif operation == ast.ConfigEntryOperation.OVERRIDE:
        config_ref.update(data_obj)
    elif operation == ast.ConfigEntryOperation.INSERT:
        config_ref.insert_with(data_obj, -1)
    else:
        config_ref.update(data_obj)
    return config_ref


def delete_item(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    collection_obj = vm.peek_nth(arg)
    is_two_var = vm.pop().value
    value = vm.pop()
    item = vm.pop()
    if isinstance(collection_obj, (obj.KCLSchemaObject, obj.KCLDictObject)):
        collection_obj.delete(item)
    elif isinstance(collection_obj, obj.KCLListObject):
        if is_two_var:
            collection_obj.remove(value)
        else:
            collection_obj.remove(item)
    else:
        kcl.report_exception(
            err_type=kcl.ErrType.EvaluationError_TYPE,
            arg_msg=f"illegal quantifier expression type '{collection_obj.type_str()}'",
        )
    return collection_obj


def schema_nop(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    """Schema nop between scheam update attributes"""
    schema_obj = vm.ctx.locals[obj.SCHEMA_SELF_VALUE_KEY]
    return schema_obj


def make_schema(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    """
    Stack Layout
    ------------
    TOS
        - 6. index signature
        - 5. decorator list
        - 4. check func
        - 3. schema_body_func
        - 2. mixin type object list
        - 1. parent_type_obj
        - 0. schema_type_obj
    """
    decorator_count, mixin_count, _ = unpack_operand(arg)
    index_signature = None
    index_signature_key_type = vm.pop().value
    if index_signature_key_type:
        index_signature = obj.KCLSchemaIndexSignatureObject(
            key_type=index_signature_key_type,
            value_type=vm.pop().value,
            key_name=vm.pop().value,
            any_other=vm.pop().value,
            value=vm.pop(),
        )
    decorators = [vm.pop() for _i in range(decorator_count)][::-1]
    schema_check_func = vm.pop()
    schema_body_func = vm.pop()
    mixins = [vm.pop() for _i in range(mixin_count)][::-1]
    parent_schema_type = vm.pop()
    if parent_schema_type.type() not in [
        obj.KCLObjectType.SCHEMA_TYPE,
        obj.KCLObjectType.NONE,
        obj.KCLObjectType.UNDEFINED,
    ]:
        kcl.report_exception(
            err_type=kcl.ErrType.EvaluationError_TYPE,
            arg_msg="illegal schema inherit object type",
        )
    self_schema_type = vm.pop()

    type_obj = obj.KCLSchemaTypeObject.new(
        self_schema_type.name,
        parent_schema_type
        if isinstance(parent_schema_type, obj.KCLSchemaTypeObject)
        else None,
        self_schema_type.protocol,
        filename=vm.get_info()[0],
        is_mixin=self_schema_type.is_mixin,
        pkgpath=self_schema_type.pkgpath,
        attr_list=self_schema_type.attr_list,
        index_signature=index_signature,
        is_relaxed=bool(index_signature),
        vm=vm,
    )

    vm.define_schema_type(
        f"{self_schema_type.pkgpath}.{self_schema_type.name}", type_obj
    )

    if isinstance(schema_body_func, obj.KCLCompiledFunctionObject):
        schema_body_func.name = self_schema_type.name
        type_obj.set_func(schema_body_func)
    else:
        assert isinstance(schema_body_func, (obj.KCLNoneObject, obj.KCLUndefinedObject))

    if isinstance(schema_check_func, obj.KCLCompiledFunctionObject):
        schema_check_func.name = self_schema_type.name
        type_obj.set_check_func(schema_check_func)
    else:
        assert isinstance(
            schema_check_func, (obj.KCLNoneObject, obj.KCLUndefinedObject)
        )

    type_obj.add_decorators(decorators)

    if len(mixins) > 0:
        if isinstance(mixins[0], obj.KCLStringObject):
            type_obj.mixins_names = [x.value for x in mixins]
        elif isinstance(mixins[0], obj.KCLSchemaTypeObject):
            type_obj.mixins = mixins
        else:
            assert False, f"make_schema: mixins[0] type: {type(mixins[0])}"
    type_obj.update_mixins(vm=vm)
    vm.push(type_obj)
    return type_obj


def build_schema(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    schema_obj = vm.pop()
    config_obj = vm.pop()
    config_meta = obj.to_python_obj(vm.pop())
    args, kwargs = _evaluator_inst.call_vars_and_keywords(arg, vm)
    # Schema type object
    if isinstance(schema_obj, obj.KCLSchemaTypeObject):
        if isinstance(config_obj, obj.KCLDictObject):
            inst = schema_obj.new_instance(config_obj, config_meta, args, kwargs, vm)
        else:
            inst = config_obj
    # Schema value object
    else:
        inst = _evaluator_inst.eval_binary_op(
            schema_obj, config_obj, Opcode.BINARY_OR, vm=vm
        )
    vm.push(inst)
    return inst


def build_schema_config(vm, _code: int, _arg: int) -> Optional[obj.KCLObject]:
    config_obj = obj.KCLSchemaConfigObject(
        value={}, operation_map={}, insert_index_map={}
    )
    vm.push(config_obj)
    return config_obj


def store_schema_config(vm, _code: int, _arg: int) -> Optional[obj.KCLObject]:
    insert_index, operation, is_nest_key = (
        vm.pop().value,
        vm.pop().value,
        vm.pop().value,
    )
    v, k = vm.pop(), vm.pop()
    config_ref = vm.peek()
    if v.type() == obj.KCLObjectType.UNPACK and k.type() in [
        obj.KCLObjectType.NONE,
        obj.KCLObjectType.UNDEFINED,
    ]:
        config_ref.append_unpack(v.unpack())
    else:
        # Deal the nest var config e.g., {a.b.c: "123"} -> {a: {b: {c: "123"}}}
        if k.type() == obj.KCLObjectType.STRING and is_nest_key:
            data_obj = obj.KCLSchemaConfigObject()
            obj_ref = data_obj
            nest_keys = k.value.split(".")
            for i, key in enumerate(nest_keys):
                obj_ref.value[key] = (
                    v if i == len(nest_keys) - 1 else obj.KCLSchemaConfigObject()
                )
                if i == len(nest_keys) - 1:
                    obj_ref.add_operation(key, operation, insert_index)
                obj_ref = obj_ref.value[key]
            operation = ast.ConfigEntryOperation.UNION
            insert_index = None
        else:
            data_obj = obj.KCLSchemaConfigObject(value={k.value: v})
            config_ref.add_operation(k.value, operation, insert_index)
        if operation is None or operation == ast.ConfigEntryOperation.UNION:
            config_ref.union_with(data_obj)
        elif operation == ast.ConfigEntryOperation.OVERRIDE:
            config_ref.update(data_obj)
        elif operation == ast.ConfigEntryOperation.INSERT:
            config_ref.insert_with(data_obj, insert_index)
        elif operation == ast.ConfigEntryOperation.UNIQUE:
            config_ref.unique_merge_with(data_obj)
        elif operation == ast.ConfigEntryOperation.UNIFICATION:
            if value_subsume(data_obj, config_ref):
                config_ref.union_with(data_obj)
            else:
                kcl.report_exception(
                    err_type=kcl.ErrType.EvaluationError_TYPE,
                    arg_msg="unification conflict",
                )
        else:
            config_ref.union_with(data_obj)
    return config_ref


def load_attr(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    attr_obj = vm.top()
    name = vm.names[arg]
    value = _evaluator_inst.load_attr(attr_obj, name)
    vm.set_top(value)
    return value


def load_builtin(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    return vm.load_builtin(arg)


def format_values(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    assert arg >= 0
    formatted_str_obj_list = []
    format_spec = vm.pop().value
    for i in range(arg):
        format_obj = vm.pop()
        format_value = _evaluator_inst.format_value(format_obj, format_spec)
        formatted_str_obj_list.append(format_value)
    for str_obj in formatted_str_obj_list[::-1]:
        vm.push(str_obj)
    return vm.peek()


def member_ship_as(vm, code: int, arg: int) -> Optional[obj.KCLObject]:
    # Pop the type object
    type_object = vm.pop()
    vm.set_top(types.type_convert(vm.top(), type_object))
    return vm.top()


def compare_op(vm, _code: int, arg: int) -> Optional[obj.KCLObject]:
    left, right = vm.pop(), vm.top()
    result = _evaluator_inst.eval_compare_op(left, right, arg, vm=vm)
    vm.set_top(result)
    return result


VM_OP_ACTIONS = {
    Opcode.BINARY_ADD: bin_add,
    Opcode.BINARY_SUBTRACT: bin_substract,
    Opcode.BINARY_MULTIPLY: bin_mul,
    Opcode.BINARY_FLOOR_DIVIDE: bin_floor_divide,
    Opcode.BINARY_TRUE_DIVIDE: bin_true_divide,
    Opcode.BUILD_SCHEMA_CONFIG: build_schema_config,
    Opcode.STORE_SCHEMA_CONFIG: store_schema_config,
    Opcode.POP_TOP: pop_top,
    Opcode.ROT_TWO: rot_two,
    Opcode.ROT_THREE: rot_three,
    Opcode.DUP_TOP: dup_top,
    Opcode.DUP_TOP_TWO: dup_top_two,
    Opcode.COPY_TOP: copy_top,
    Opcode.NOP: nop,
    Opcode.UNARY_POSITIVE: unary_pos,
    Opcode.UNARY_NEGATIVE: unary_neg,
    Opcode.UNARY_NOT: unary_not,
    Opcode.UNARY_INVERT: unary_invert,
    Opcode.BINARY_POWER: bin_power,
    Opcode.BINARY_MODULO: bin_mod,
    Opcode.BINARY_SUBSCR: bin_subscr,
    Opcode.INPLACE_FLOOR_DIVIDE: inplace_floor_div,
    Opcode.INPLACE_TRUE_DIVIDE: inplace_true_div,
    Opcode.STORE_MAP: store_map,
    Opcode.INPLACE_ADD: inplace_add,
    Opcode.INPLACE_SUBTRACT: inplace_sub,
    Opcode.INPLACE_MULTIPLY: inplace_mul,
    Opcode.INPLACE_MODULO: inplace_mod,
    Opcode.STORE_SUBSCR: store_subscr,
    Opcode.BINARY_LSHIFT: bin_lshift,
    Opcode.BINARY_RSHIFT: bin_rshift,
    Opcode.BINARY_AND: bin_and,
    Opcode.BINARY_XOR: bin_xor,
    Opcode.BINARY_OR: bin_or,
    Opcode.BINARY_LOGIC_AND: bin_logic_and,
    Opcode.BINARY_LOGIC_OR: bin_logic_or,
    Opcode.INPLACE_POWER: inplace_pow,
    Opcode.GET_ITER: get_iter,
    Opcode.PRINT_EXPR: print_expr,
    Opcode.EMIT_EXPR: emit_expr,
    Opcode.INPLACE_LSHIFT: inplace_lshift,
    Opcode.INPLACE_RSHIFT: inplace_rlshift,
    Opcode.INPLACE_AND: inplace_and,
    Opcode.INPLACE_XOR: inplace_xor,
    Opcode.INPLACE_OR: inplace_or,
    Opcode.RETURN_VALUE: return_value,
    Opcode.RETURN_LAST_VALUE: return_last_value,
    Opcode.STORE_NAME: store_name,
    Opcode.UNPACK_SEQUENCE: unpack_sequence,
    Opcode.FOR_ITER: for_iter,
    Opcode.STORE_ATTR: store_attr,
    Opcode.STORE_GLOBAL: store_global,
    Opcode.LOAD_CONST: load_const,
    Opcode.LOAD_NAME: load_name,
    Opcode.BUILD_LIST: build_list,
    Opcode.BUILD_MAP: build_map,
    Opcode.LOAD_ATTR: load_attr,
    Opcode.IMPORT_NAME: import_name,
    Opcode.JUMP_FORWARD: jump_forward,
    Opcode.JUMP_IF_FALSE_OR_POP: jump_if_false_or_pop,
    Opcode.JUMP_IF_TRUE_OR_POP: jump_if_true_or_pop,
    Opcode.JUMP_ABSOLUTE: jump_absolute,
    Opcode.POP_JUMP_IF_FALSE: pop_jump_if_false,
    Opcode.POP_JUMP_IF_TRUE: pop_jump_if_true,
    Opcode.LOAD_GLOBAL: load_global,
    Opcode.RAISE_VARARGS: raise_varargs,
    Opcode.RAISE_CHECK: raise_check,
    Opcode.LOAD_LOCAL: load_local,
    Opcode.STORE_LOCAL: store_local,
    Opcode.LOAD_FREE: load_free,
    Opcode.CALL_FUNCTION: call_function,
    Opcode.MAKE_FUNCTION: make_function,
    Opcode.BUILD_SLICE: build_slice,
    Opcode.LOAD_CLOSURE: load_closure,
    Opcode.MAKE_CLOSURE: make_closure,
    Opcode.LIST_APPEND: list_append,
    Opcode.MAP_ADD: map_add,
    Opcode.DELETE_ITEM: delete_item,
    Opcode.MAKE_SCHEMA: make_schema,
    Opcode.BUILD_SCHEMA: build_schema,
    Opcode.SCHEMA_ATTR: schema_attr,
    Opcode.LOAD_BUILT_IN: load_builtin,
    Opcode.COMPARE_OP: compare_op,
    Opcode.MAKE_DECORATOR: make_decorator,
    Opcode.SCHEMA_LOAD_ATTR: schema_load_attr,
    Opcode.SCHEMA_UPDATE_ATTR: schema_update_attr,
    Opcode.SCHEMA_NOP: schema_nop,
    Opcode.FORMAT_VALUES: format_values,
    Opcode.MEMBER_SHIP_AS: member_ship_as,
    Opcode.DEBUG_STACK: debug_stack,
    Opcode.DEBUG_LOCALS: debug_locals,
    Opcode.DEBUG_GLOBALS: debug_globals,
    Opcode.DEBUG_NAMES: debug_names,
}
