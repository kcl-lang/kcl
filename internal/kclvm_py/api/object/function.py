# Copyright 2021 The KCL Authors. All rights reserved.

from abc import abstractmethod
from typing import Callable, List, Optional
from dataclasses import dataclass

from .object import (
    ANT_TYPE_STR,
    KCLObject,
    KCLListObject,
    KCLObjectType,
    KCLBaseTypeObject,
    KCLTypeKind,
    to_python_obj,
    to_kcl_obj,
)

BUILTIN_KCL_OBJ_FUNCTIONS = [
    "typeof",
]


@dataclass
class KWArg:
    """KCL function call arguments

    Parameters
    ----------
    - name: str
    - value: KCLObject
    """

    name: KCLObject
    value: KCLObject


@dataclass
class Parameter:
    """KCL function definition parameter

    Parameters
    ----------
    - name: str
        The argument name
    - value: KCLObject
        Value is the default value to use if the argument is not provided.
        If no default is specified then value is None.
    - type_annotation: str
        The argument type annotation.
    """

    name: str = None
    value: Optional[KCLObject] = None
    type_annotation: Optional[str] = None
    type: Optional[KCLBaseTypeObject] = None

    def param_doc(self) -> str:
        _type_str = self.type.type_str() if self.type else "any"
        _value_doc = ""
        if self.value:
            _value_str = (
                f'"{self.value.value}"'
                if isinstance(self.value.value, str)
                else f"{self.value.value}"
            )
            _value_doc = f"={_value_str}"
        return f"{self.name}: {_type_str}{_value_doc}"


@dataclass
class KCLFunctionObject(KCLObject):
    name: str

    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.FUNCTION

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "function"

    @abstractmethod
    def call(self, args: List[KCLObject], kwargs: List[KWArg], vm=None) -> KCLObject:
        pass


@dataclass
class KCLBuiltinFunctionObject(KCLFunctionObject):
    function: Callable

    def call(self, args: List[KCLObject], kwargs: List[KWArg], vm=None) -> KCLObject:
        if not self.function:
            raise Exception("invalid kcl function object")

        if self._is_kcl_obj_builtin():
            return to_kcl_obj(
                self.function(*args, **{kw.name.value: kw.value for kw in kwargs})
            )

        return to_kcl_obj(
            self.function(
                *to_python_obj(args),
                **to_python_obj({kw.name.value: kw.value for kw in kwargs}),
            )
        )

    def _is_kcl_obj_builtin(self) -> bool:
        return self.name in BUILTIN_KCL_OBJ_FUNCTIONS


@dataclass
class KCLMemberFunctionObject(KCLFunctionObject):
    obj: KCLObject

    def call(self, args: List[KCLObject], kwargs: List[KWArg], vm=None) -> KCLObject:
        return to_kcl_obj(
            self.obj.call_member_method(
                self.name,
                *to_python_obj(args),
                **to_python_obj({kw.name.value: kw.value for kw in kwargs}),
            )
        )


@dataclass
class KCLCompiledFunctionObject(KCLFunctionObject):
    instructions: list = None
    names: list = None
    constants: list = None
    num_parameters: int = 0
    num_locals: int = 0
    params: List[Parameter] = None
    pkgpath: str = None
    closure: KCLListObject = None


@dataclass
class KCLClosureObject(KCLObject):
    name: str = None
    parameters: List[str] = None

    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.CLOSURE

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "closure"


@dataclass
class KCLFunctionTypeObject(KCLBaseTypeObject):
    name: str
    params: List[Parameter]
    self_type: Optional[KCLBaseTypeObject]
    return_type: KCLBaseTypeObject
    doc: Optional[str] = None
    kwonlyargs_index: int = None
    is_variadic: bool = False

    def type_str(self) -> str:
        return "function(({}) -> {})".format(
            ", ".join(
                [
                    p.type.type_str() if p.type else ANT_TYPE_STR
                    for p in self.params or []
                ]
            ),
            self.return_type.type_str() if self.return_type else ANT_TYPE_STR,
        )

    def type_kind(self) -> str:
        return KCLTypeKind.FuncKind
