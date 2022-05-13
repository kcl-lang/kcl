# Copyright 2021 The KCL Authors. All rights reserved.

from typing import Optional, Union, List, Callable
from abc import ABCMeta, abstractmethod

import kclvm.kcl.info as kcl_info
import kclvm.kcl.error as kcl_error

from .common import (
    LIST_TYPE_NAME,
    DICT_TYPE_NAME,
    SCHEMA_TYPE_NAME,
)

SCHEMA_SETTINGS_ATTR_NAME = "__settings__"
SELECTOR_ERROR = "SelectorError"
SELECTOR_EMPTY_VAR_ERROR_MSG = "selector expression variable can't be empty"
SELECTOR_INVALID_VAR_TYPE_ERROR_MSG = (
    "invalid selector expression variable type, expected list, dict and Schema"
)
SELECTOR_INVALID_EXPR_ERROR_MSG = "invalid selector expression"
SELECTOR_INVALID_USE_ERROR_MSG = "{} variable can't be used with {} selector expression"
SELECTOR_INVALID_CONDITION_ERROR_MSG = "invalid selector expression lambda condition"


SelectorVar = Union[list, dict]


class Selector(metaclass=ABCMeta):
    """
    Selector expression interface
    """

    def __init__(self, data: SelectorVar) -> None:
        self.data: SelectorVar = data

    @abstractmethod
    def get_all(self) -> Optional[SelectorVar]:
        """
        Get all child items form self.data
        """
        pass

    @abstractmethod
    def get_by_index(self, index: int) -> Optional[SelectorVar]:
        """
        Get all child items by index
        """
        pass

    @abstractmethod
    def get_by_keys(self, keys: List[str]) -> Optional[SelectorVar]:
        """
        Get all child items form self.data when child key is in keys
        """
        pass

    @abstractmethod
    def get_item_by_condition(self, condition: Callable) -> Optional[SelectorVar]:
        """
        Get all child items form self.data when child meets the condition
        """
        pass

    def get_var(self) -> SelectorVar:
        """
        Get data variable
        """
        return self.data

    def select(
        self,
        is_all: bool = False,
        index: Optional[int] = None,
        keys: List[str] = None,
        condition: Callable = None,
    ) -> Optional[SelectorVar]:
        if is_all:
            return self.get_all()
        if index is not None:
            return self.get_by_index(index)
        if keys:
            return self.get_by_keys(keys)
        if condition and callable(condition):
            return self.get_item_by_condition(condition)
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.SelectorError_TYPE,
            arg_msg=SELECTOR_INVALID_EXPR_ERROR_MSG,
        )

    @staticmethod
    def _handle_collection(
        result: Optional[Union[list, dict]]
    ) -> Optional[Union[list, dict]]:
        """
        If the result len is zero, return None, else return itself
        """
        return result if result else None


class ListSelector(Selector):
    """
    List selector expression inherited from Selector
    """

    def __init__(self, data: SelectorVar) -> None:
        super().__init__(data)

    def get_all(self) -> SelectorVar:
        """
        Get all child items form self.data
        """
        return self.get_var()

    def get_by_index(self, index: int) -> Optional[SelectorVar]:
        """
        Get all child items by index
        """
        var = self.get_var()
        return var[index] if -len(var) <= index < len(var) else None

    def get_by_keys(self, keys: List[str]) -> SelectorVar:
        """
        Get all child items form self.data when child key is in keys
        """
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.SelectorError_TYPE,
            arg_msg=SELECTOR_INVALID_USE_ERROR_MSG.format(
                LIST_TYPE_NAME, DICT_TYPE_NAME
            ),
        )

    def get_item_by_condition(self, condition: Callable) -> Optional[SelectorVar]:
        """
        Get all child items form self.data when child meets the condition
        """
        var = self.get_var()
        if condition.__code__.co_argcount == 1:
            return self._handle_collection([v for v in var if condition(v)])
        elif condition.__code__.co_argcount == 2:
            return self._handle_collection(
                [v for i, v in enumerate(var) if condition(i, v)]
            )
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.SelectorError_TYPE,
            arg_msg=SELECTOR_INVALID_CONDITION_ERROR_MSG,
        )


class DictSelector(Selector):
    """
    List selector expression inherited from Selector
    """

    def __init__(self, data: SelectorVar) -> None:
        super().__init__(data)

    def get_all(self) -> SelectorVar:
        """
        Get all child items form self.data
        """
        return self.get_var()

    def get_by_index(self, index: int) -> SelectorVar:
        """
        Get all child items by index
        """
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.SelectorError_TYPE,
            arg_msg=SELECTOR_INVALID_USE_ERROR_MSG.format(
                DICT_TYPE_NAME, LIST_TYPE_NAME
            ),
        )

    def get_by_keys(self, keys: List[str]) -> Optional[SelectorVar]:
        """
        Get all child items form self.data when child key is in keys
        """
        var = self.get_var()
        return self._handle_collection({k: var[k] for k in var if k in keys})

    def get_item_by_condition(self, condition: Callable) -> Optional[SelectorVar]:
        """
        Get all child items form self.data when child meets the condition
        """
        var = self.get_var()
        if condition.__code__.co_argcount == 1:
            return self._handle_collection({k: var[k] for k in var if condition(k)})
        elif condition.__code__.co_argcount == 2:
            return self._handle_collection(
                {k: v for k, v in var.items() if condition(k, v)}
            )
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.SelectorError_TYPE,
            arg_msg=SELECTOR_INVALID_CONDITION_ERROR_MSG,
        )


class SchemaSelector(Selector):
    """
    Schema selector expression inherited from Selector
    """

    def __init__(self, data: SelectorVar) -> None:
        super().__init__(data)

    def get_all(self) -> Optional[SelectorVar]:
        """
        Get all child items form self.data
        """
        var = self.get_var()
        return self._handle_collection(
            {
                kcl_info.demangle(k): var[k]
                for k in var
                if kcl_info.ismangled(k) and SCHEMA_SETTINGS_ATTR_NAME not in k
            }
        )

    def get_by_index(self, index: int) -> SelectorVar:
        """
        Get all child items by index
        """
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.SelectorError_TYPE,
            arg_msg=SELECTOR_INVALID_USE_ERROR_MSG.format(
                SCHEMA_TYPE_NAME, LIST_TYPE_NAME
            ),
        )

    def get_by_keys(self, keys: List[str]) -> Optional[SelectorVar]:
        """
        Get all child items form self.data when child key is in keys
        """
        data = self.get_all()
        return self._handle_collection({k: data[k] for k in data if k in keys})

    def get_item_by_condition(self, condition: Callable) -> Optional[SelectorVar]:
        """
        Get all child items form self.data when child meets the condition
        """
        var = self.get_var()
        if condition.__code__.co_argcount == 1:
            return self._handle_collection(
                {
                    kcl_info.demangle(k): var[k]
                    for k in var
                    if kcl_info.ismangled(k) and condition(kcl_info.demangle(k))
                }
            )
        elif condition.__code__.co_argcount == 2:
            return self._handle_collection(
                {
                    kcl_info.demangle(k): v
                    for k, v in var.items()
                    if kcl_info.ismangled(k) and condition(kcl_info.demangle(k), v)
                }
            )
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.SelectorError_TYPE,
            arg_msg=SELECTOR_INVALID_CONDITION_ERROR_MSG,
        )


class SelectorFactory:
    """
    Factory class to build selector
    """

    @staticmethod
    def get(var: SelectorVar) -> Selector:
        """
        Get selector class using 'var'
        """

        def is_kcl_schema(obj):
            from kclvm.api.object import KCLSchemaObject

            return isinstance(obj, KCLSchemaObject)

        if isinstance(var, list):
            return ListSelector(var)
        if isinstance(var, dict):
            return DictSelector(var)
        if is_kcl_schema(var):
            return SchemaSelector(var)
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.SelectorError_TYPE,
            arg_msg=SELECTOR_INVALID_VAR_TYPE_ERROR_MSG,
        )


def select(
    var: SelectorVar,
    is_all: bool = False,
    index: Optional[int] = None,
    keys: List[str] = None,
    condition: Callable = None,
):
    """
    Use the selector expression to filter out the child elements of 'var'
    and return their references
    """
    if var is None:
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.SelectorError_TYPE,
            arg_msg=SELECTOR_EMPTY_VAR_ERROR_MSG,
        )
    return SelectorFactory.get(var).select(is_all, index, keys or [], condition)
