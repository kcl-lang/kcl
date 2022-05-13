"""
The `check_utils` file mainly contains some methods for defensive programming.

Method `PreCheck` can be used for pre-checking the method,
mainly to verify whether the incoming parameters of the method meet conditions.

Method `PostCheck` can be used for post-checking the method,
mainly to verify whether the return of the method meets conditions.

Method `PostSimpleExprCheck` can be used for post-checking the method,
Compared with `PostCheck`, `PostSimpleExprCheck` supports verifying
some simple relationship between the return value and the input parameters

For example：

# Check whether the type of incoming parameter "a" is int
@PreCheck((lambda v: isinstance(v, int)), "a")
# Check whether the type of incoming parameter "b" is int
@PreCheck((lambda v: isinstance(v, int)), "b")
# Check whether the type of return value is int
@PostCheck((lambda v: isinstance(v, int)))
# Check whether the return value is equal to the sum of input parameters
@PostSimpleExprCheck((lambda inputs, result: result == inputs["a"] + inputs["b"]), ["a", "b"])
def add(a, b):
    return a + b

Class `CheckRules` and some other global methods are built-in check rules
that can be used in `PreCheck` and `PostCheck`.

:copyright: Copyright 2020 The KCL Authors. All rights reserved.
"""
import locale
import typing
import functools
from inspect import Signature
from typing import Callable, Any, List

# CHECK_MODE is a switch,
# you can return the parameters directly by turning the CHECK_MODE = False

CHECK_MODE = False


class CheckRules:
    @staticmethod
    def check_list_len_equal(all_lists: list):
        if not CHECK_MODE:
            return
        assert all_lists
        assert isinstance(all_lists, list)
        assert all(
            result is True
            for result in [isinstance(list_inner, list) for list_inner in all_lists]
        )
        assert all(
            result == len(all_lists[0])
            for result in [len(list_inner) for list_inner in all_lists]
        )

    @staticmethod
    def check_locale(lang: str):
        if not CHECK_MODE:
            return True
        LOCALE_LIST = list(locale.locale_alias.keys())
        if not lang or not isinstance(lang, str) or lang not in LOCALE_LIST:
            return False
        return True

    @staticmethod
    def check_type_not_none(item, *tpes) -> bool:
        if CHECK_MODE:
            return item is not None and isinstance(item, tpes)
        return True

    @staticmethod
    def check_type_allow_none(item, *tpes) -> bool:
        if CHECK_MODE:
            return item is None or isinstance(item, tpes)
        return True

    @staticmethod
    def check_list_item_type_allow_none(item, *tpes) -> bool:
        if CHECK_MODE:
            check_all_allow_none(list, item, *tpes)
        return True

    @staticmethod
    def check_int_range_allow_none(target: int, low: int, high: int) -> bool:
        if CHECK_MODE:
            if target is None:
                return True
            check_type_not_none(target, int)
            check_type_not_none(low, int)
            check_type_not_none(high, int)
            return target in range(low, high)
        else:
            return True

    @staticmethod
    def check_str_len_not_none(target: str, length: int) -> bool:
        if CHECK_MODE:
            check_type_not_none(target, str)
            check_type_not_none(length, int)
            return len(target) == length
        else:
            return True

    @staticmethod
    def check_str_len_allow_none(target: str, length: int) -> bool:
        if CHECK_MODE:
            if target is None:
                return True
            check_type_not_none(target, str)
            check_type_not_none(length, int)
            return len(target) == length
        else:
            return True


def check_allow_none(node, tpe):
    if node and CHECK_MODE:
        assert isinstance(node, tpe)
    return typing.cast(tpe, node)


def check_all_allow_none(set_tpe: typing.Type, nodes, *item_tpes):
    if nodes and CHECK_MODE:
        assert isinstance(nodes, set_tpe)
        assert isinstance(nodes, (list, tuple)) and all(
            isinstance(item, item_tpes) for item in nodes
        )
    return typing.cast(set_tpe, nodes)


def check_not_none(node, *tpes):
    if CHECK_MODE:
        assert node and isinstance(node, tpes)
    return typing.cast(tpes, node)


def check_all_not_none(set_tpe: typing.Type, nodes, *item_tpes):
    if CHECK_MODE:
        assert nodes and isinstance(nodes, set_tpe)
        assert isinstance(nodes, (list, tuple)) and all(
            isinstance(item, item_tpes) for item in nodes
        )
    return typing.cast(set_tpe, nodes)


def check_type_allow_none(node, *tpes):
    if node and CHECK_MODE:
        assert isinstance(node, tpes)
    return node


def check_type_not_none(node, *tpes):
    if CHECK_MODE:
        assert node is not None and isinstance(node, tpes)
    return node


def alert_internal_bug():
    if CHECK_MODE:
        assert False, "Here is unreachable unless a bug occurs"


def PreCheck(condition: Callable[[Any], bool], param_name: str, param_pos: int = None):
    def conditioner(func):
        @functools.wraps(func)
        def check_condition(*args, **kwargs):
            if not CHECK_MODE:
                return func(*args, **kwargs)
            check_type_not_none(condition, Callable)
            check_type_not_none(func, Callable)
            check_type_not_none(CHECK_MODE, bool)
            check_type_not_none(param_name, str)
            check_type_allow_none(param_pos, int)
            param_names_list = [
                i[0]
                for i in Signature.from_callable(func).parameters.items()
                if len(i) > 0
            ]
            if param_name not in param_names_list:
                raise AssertionError(
                    f"Pre-Condition failed: "
                    f"There is no parameter named {param_name} in function {func.__name__}. "
                    f"The function parameters list: {param_names_list}."
                )
            try:
                param_value = kwargs[
                    param_name
                ]  # if the param in kwargs for the function
            except KeyError:
                if not CheckRules.check_int_range_allow_none(
                    param_pos, 0, len(param_names_list)
                ):
                    raise AssertionError(
                        f"Pre-Condition failed: param_pos: {param_pos} is out of range. "
                        f"There are only {len(param_names_list)} parameters in {func.__name__}"
                    )
                if param_names_list.index(param_name) < len(
                    args
                ):  # if the param in args for the function
                    param_value = (
                        args[param_pos]
                        if param_pos
                        else args[param_names_list.index(param_name)]
                    )
                else:
                    param_value = None

            if condition(param_value):
                return func(*args, **kwargs)
            else:
                raise AssertionError(
                    f"Pre-Condition failed: {func.__name__}({param_name} = {param_value}), "
                    f"Check Condition: {condition.__name__}"
                )

        return check_condition

    return conditioner


def PostCheck(condition: Callable[[Any], bool]):
    def conditioner(func):
        @functools.wraps(func)
        def check_condition(*args, **kwargs):
            if not CHECK_MODE:
                return func(*args, **kwargs)
            check_type_not_none(condition, Callable)
            check_type_not_none(func, Callable)
            check_type_not_none(CHECK_MODE, bool)
            result = func(*args, **kwargs)
            if condition(result):
                return result
            else:
                raise AssertionError(
                    f"Post-Condition failed: {func.__name__} with returned {result},"
                    f"Check Condition: {condition.__name__}"
                )

        return check_condition

    return conditioner


def PostSimpleExprCheck(
    condition: Callable[[Any, Any], bool], dependent_params: List[str]
):
    def conditioner(func):
        @functools.wraps(func)
        def check_condition(*args, **kwargs):
            if not CHECK_MODE:
                return func(*args, **kwargs)
            check_type_not_none(condition, Callable)
            check_type_not_none(func, Callable)
            check_type_not_none(CHECK_MODE, bool)
            check_all_not_none(list, dependent_params, str)
            sig_params = {}
            param_names_list = [
                i[0]
                for i in Signature.from_callable(func).parameters.items()
                if len(i) > 0
            ]
            for (k, v) in Signature.from_callable(func).parameters.items():
                sig_params[k] = (k, v)
            inputs = {}

            for param in dependent_params:
                try:
                    inputs[param] = kwargs[
                        param
                    ]  # if the param in kwargs for the function
                except KeyError:
                    try:
                        inputs[param] = args[  # if the param in args for the function
                            param_names_list.index(sig_params[param][0])
                        ]
                    except IndexError:
                        assert (
                            len(sig_params[param]) > 0
                        )  # if the param in the function has default value
                        inputs[param] = sig_params[param][1].default
                    except KeyError:
                        raise AssertionError(
                            f"Post-Condition failed: "
                            f"There is no parameter named {param} in function {func.__name__}. "
                            f"The function parameters list: {param_names_list}."
                        )

            result = func(*args, **kwargs)
            if condition(inputs, result):
                return result
            else:
                raise AssertionError(
                    f"Post-condition failed: {func.__name__} with inputs: {inputs}, returned {result}"
                )

        return check_condition

    return conditioner
