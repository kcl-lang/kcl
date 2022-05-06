# Copyright 2021 The KCL Authors. All rights reserved.

from typing import Union, Tuple, Optional, List

import kclvm.kcl.error as kcl_error
import kclvm.config
from kclvm.internal.util import dotdict

from .common import union
from .selector import select

SELECT_ALL_SYMBOL = "*"
SELECT_INDEX_LEFT_SYMBOL = "["
SELECT_INDEX_RIGHT_SYMBOL = "]"
SELECT_KEYS_LEFT_SYMBOL = "{"
SELECT_KEYS_RIGHT_SYMBOL = "}"
SELECT_KEYS_SPLIT_SYMBOL = ","


def build_selector_index() -> dict:
    """Build path selector index map"""

    return _build_selector_index(kclvm.config.path_selector)


def _build_selector_index(path_selector: List[List[str]]):
    """
    Build selector index with given path selector: [pkg, name.attr1.attr2]

    - single element: name.name1.name2
    - elements: name.{name1,name2}.name, notice that there is no space between name1 and Name2
    - all elements: name.*.name
    - list indices: name.[0].name

    :param filename:
    :param path_selector:
    :return: index with format: {name: {attr1: {attr2: ... attrn: {}}}}
    """
    identifiers = list(
        map(
            lambda select_item: list(map(eval_py_data, select_item[1].split("."))),
            path_selector,
        )
    )
    select_index = dotdict()
    for identifier in identifiers:
        index = select_index
        for name in identifier:
            if not index.get(name):
                index[name] = dotdict()
            index = index[name]
    return select_index


def eval_py_data(s):
    if isinstance(s, str) and s.isnumeric():
        import ast

        return ast.literal_eval(s)
    return s


def is_selector_mode() -> bool:
    """Mark whether is path selector mode"""
    return len(kclvm.config.path_selector) > 0


def parse_selector(
    select_value: Union[str, int, float]
) -> Tuple[bool, Optional[int], list]:
    """Parse input selector string to selector conditions

    Returns:
        is_all: bool
        index: Optional[int]
        keys: List[str]
    """
    # If the selector is numeric, it is a key
    if isinstance(select_value, (int, float)):
        return False, None, [select_value]
    # Case 1: all element selector a.*
    is_all = len(select_value) == 1 and select_value[0] == SELECT_ALL_SYMBOL
    index = None
    keys = []
    # Case 2: index selector a.[0]
    is_may_index = (
        len(select_value) > 2
        and select_value[0] == SELECT_INDEX_LEFT_SYMBOL
        and select_value[-1] == SELECT_INDEX_RIGHT_SYMBOL
    )
    if is_may_index:
        try:
            index = int(select_value[1:-1])
        except Exception:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.CompileError_TYPE,
                arg_msg="invalid path selector value {}".format(select_value),
            )
    is_may_keys = (
        len(select_value) > 2
        and select_value[0] == SELECT_KEYS_LEFT_SYMBOL
        and select_value[-1] == SELECT_KEYS_RIGHT_SYMBOL
    )
    # Case 3: keys selector a.{key1, key2}
    if is_may_keys:
        keys += select_value[1:-1].split(SELECT_KEYS_SPLIT_SYMBOL)
    # Case 4: single selector a.b.c
    if not is_all and index is None and not keys:
        keys.append(select_value)
    return is_all, index, keys


def select_instance_attributes(instance, attrs):
    """
    Select attributes from instance with dot selector, like a.b.c

    - single element: name.name1.name2
    - elements: name.{name1, name2}.name
    - all elements: name.*.name
    - list indices: name.[0].name

    :param instance: instance to select attributes
    :param attrs: attributes used to select, format: {attr1: {attr2: ... attrn: {}}}
    :return:
    """

    def select_instance_attribute(instance, attrs):
        selected = {}
        for attr in attrs:
            attr_name = attr
            # 1. Parse path selector including [0], *, {key1, key2, ...} and key
            is_all, index, keys = parse_selector(attr_name)
            # 2. Select value according path selector value
            select_result = select(instance, is_all, index, keys)
            if not select_result:
                return None
            select_result = select_result[keys[0]] if len(keys) == 1 else select_result
            # 3. Sub select result if more attr to select
            sub_select_result = {}
            # 4. Get the sub path select result
            if attrs.get(attr):
                sub_select_result = select_instance_attribute(
                    select_result,
                    attrs[attr],
                )
            # Dict/Schema selector
            final_result = sub_select_result if sub_select_result else select_result
            if not isinstance(instance, list):
                if len(keys) == 1:
                    selected[keys[0]] = final_result
                else:
                    selected = union(selected, final_result)
            # List selector
            else:
                selected = final_result
        return selected

    # Select self
    if not attrs:
        return instance

    result = select_instance_attribute(instance, attrs)
    # No select result, return None
    return result or None
