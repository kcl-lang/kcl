# Copyright 2021 The KCL Authors. All rights reserved.

import re

import kclvm.kcl.error as kcl_error

DICT_TYPE_NAME = "dict"
LIST_TYPE_NAME = "list"
SCHEMA_TYPE_NAME = "Schema"
CLASS_TYPE_TMPL = "<class '{}'>"
builtin_types = ["str", "int", "float", "bool"]


def isdicttype(expectedtype):
    if isinstance(expectedtype, str) and len(expectedtype) >= 2:
        return expectedtype[0] == "{" and expectedtype[-1] == "}"
    else:
        return False


def islisttype(expectedtype):
    if isinstance(expectedtype, str) and len(expectedtype) >= 2:
        return expectedtype[0] == "[" and expectedtype[-1] == "]"
    else:
        return False


def separate_kv(expectedtype):
    stack = ""
    n = 0
    try:
        for c in expectedtype:
            if c == "[" or c == "{":
                stack += c
            elif c == "]":
                if stack[-1] != "[":
                    raise
                stack = stack[:-1]
            elif c == "}":
                if stack[-1] != "{":
                    raise
                stack = stack[:-1]
            elif c == ":":
                if len(stack) != 0:
                    raise
                return expectedtype[:n], expectedtype[n + 1 :]
            n += 1
    except Exception:
        return None, None
    return "", ""


def dereferencetype(expectedtype):
    if (
        len(expectedtype) > 1
        and (expectedtype[0] == "[" and expectedtype[-1] == "]")
        or (expectedtype[0] == "{" and expectedtype[-1] == "}")
    ):
        return expectedtype[1:-1]
    return expectedtype


def split_type_union(type_union: str):
    """
    Split the union type e.g. 'A|B|C' -> ['A', 'B', 'C'], do not split '|' in dict and list
    """
    i = 0
    s_index = 0
    stack = []
    types = []
    while i < len(type_union):
        c = type_union[i]
        if c == "|" and len(stack) == 0:
            types.append(type_union[s_index:i])
            s_index = i + 1
        # List/Dict type
        if c == "[" or c == "{":
            stack.append(c)
        # List/Dict type
        if c == "]" or c == "}":
            stack.pop()
        # String literal type
        if c == '"':
            matched = re.match(r'"(?!"").*?(?<!\\)(\\\\)*?"', type_union[i:])
            if matched:
                i += matched.span()[1] - 1
        elif c == "'":
            matched = re.match(r"'(?!'').*?(?<!\\)(\\\\)*?'", type_union[i:])
            if matched:
                i += matched.span()[1] - 1
        i += 1
    if len(stack) != 0:
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.EvaluationError_TYPE,
            arg_msg="invalid bracket matching {}".format(stack[-1]),
        )
    types.append(type_union[s_index:])
    return types


def is_type_union(tpe: str) -> bool:
    """Whether a type string is a union type string, e.g. A|B|C,
    and detect '|' in type string except '|' in dict or list.
    """
    stack = []
    i = 0
    while i < len(tpe or ""):
        c = tpe[i]
        if c == "|" and not stack:
            return True
        if c in "[{":
            stack.append(c)
        if c in "]}":
            stack.pop()
        if c == '"':
            matched = re.match(r'"(?!"").*?(?<!\\)(\\\\)*?"', tpe[i:])
            if matched:
                i += matched.span()[1] - 1
        elif c == "'":
            matched = re.match(r"'(?!'').*?(?<!\\)(\\\\)*?'", tpe[i:])
            if matched:
                i += matched.span()[1] - 1
        i += 1
    return False


def is_builtin_type(tpe_str):
    return tpe_str in builtin_types


def get_tpes_str(tpes):
    if not tpes:
        return tpes
    tpes_str = ""
    idx = 0
    for tpe in tpes:
        if idx < len(tpes) - 1:
            tpes_str += "{}, ".format(demangle_type(tpe))
        else:
            tpes_str += "{}".format(demangle_type(tpe))
        idx += 1
    return tpes_str


def get_class_name(tpe_str):
    if "class" in tpe_str:
        return re.match(r"<class \'(.*)\'>", tpe_str).group(1)
    else:
        return tpe_str


def get_builtin_type(tpe_str):
    if "kclvm_runtime.builtins" in tpe_str:
        return re.match(r"kclvm_runtime.builtins.(.*)", tpe_str).group(1)
    else:
        return tpe_str


def demangle_type(tpe: str, var=None):
    if not tpe:
        return tpe
    if isdicttype(tpe):
        key_tpe, value_tpe = separate_kv(dereferencetype(tpe))
        return "{{{}:{}}}".format(demangle_type(key_tpe), demangle_type(value_tpe))
    elif islisttype(tpe):
        return "[{}]".format(demangle_type(dereferencetype(tpe)))
    else:
        return get_builtin_type(tpe)


def do_union(obj, delta):
    """
    Union delta to obj recursively
    """
    obj_tpe = get_class_name(str(type(obj)))
    delta_tpe = get_class_name(str(type(delta)))
    if isinstance(obj, list):
        if not isinstance(delta, list):
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.TypeError_Runtime_TYPE,
                arg_msg="union failure, expect list, got {}".format(
                    demangle_type(delta_tpe, delta)
                ),
            )
        length = len(obj) if len(obj) > len(delta) else len(delta)
        result_list = obj
        for idx in range(length):
            if idx >= len(obj):
                result_list.append(delta[idx])
            elif idx < len(delta):
                result_list[idx] = union(result_list[idx], delta[idx])
        return result_list
    if isinstance(obj, dict):
        if not isinstance(delta, dict):
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.TypeError_Runtime_TYPE,
                arg_msg="union failure, expect dict, got {}".format(
                    demangle_type(delta_tpe, delta)
                ),
            )
        result_dict = obj
        for k in delta:
            if k not in obj:
                result_dict[k] = delta[k]
            else:
                result_dict[k] = union(obj[k], delta[k])
        return result_dict
    if type(obj) != type(delta):
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.TypeError_Runtime_TYPE,
            arg_msg="union failure, expect {}, got {}".format(
                demangle_type(obj_tpe), demangle_type(delta_tpe, delta)
            ),
        )
    return delta


def union(obj, delta, or_mode=False):
    if obj is None:
        return delta
    if delta is None:
        return obj
    if isinstance(obj, (list, dict)) or isinstance(delta, (list, dict)):
        return do_union(obj, delta)
    if or_mode:
        return obj | delta
    else:
        return obj if delta is None else delta
