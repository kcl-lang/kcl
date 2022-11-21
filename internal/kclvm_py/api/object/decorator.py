# Copyright 2021 The KCL Authors. All rights reserved.

from typing import List, Any
from dataclasses import dataclass, field

from kclvm.api.object.internal import decorator_factory, Decorator, DecoratorTargetType

from .object import (
    KCLObject,
    KCLObjectType,
    to_python_obj,
    to_kcl_obj,
)
from .function import (
    KCLFunctionObject,
    KWArg,
)


@dataclass
class KCLDecoratorObject(KCLFunctionObject):
    target: DecoratorTargetType
    name: str
    key: str = field(default_factory=lambda: "")
    value: Any = field(default_factory=lambda: None)
    decorator: Decorator = field(default_factory=lambda: None)

    def resolve(self, args: List[KCLObject], kwargs: List[KWArg]):
        """Build a internal decorator object"""
        args = to_python_obj(args)
        kwargs = to_python_obj({kw.name.value: kw.value for kw in kwargs})
        self.decorator = decorator_factory.get(self.name, self.target, *args, **kwargs)
        return self

    def call(
        self, args: List[KCLObject], kwargs: List[KWArg], vm=None, key=None, value=None
    ) -> KCLObject:
        """Decorator run"""
        self.key = to_python_obj(key)
        self.value = to_python_obj(value)
        return to_kcl_obj(
            self.decorator.run(
                self.key,
                self.value if self.target == DecoratorTargetType.ATTRIBUTE else None,
            )
        )

    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.DECORATOR

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "decorator"
