# Copyright 2021 The KCL Authors. All rights reserved.

import typing

from .object import (
    KCLObject,
    to_python_obj,
    to_kcl_obj,
)
import kclvm.api.object.internal as internal


class KCLResult:
    def __init__(
        self, m: typing.Dict[str, KCLObject], filename: typing.Optional[str] = None
    ):
        self.m: typing.Dict[str, KCLObject] = m
        self.filename: str = filename

    def __str__(self) -> str:
        return f"{self.m}"

    def filter_by_path_selector(
        self, to_kcl: bool = True
    ) -> typing.Dict[str, KCLObject]:
        if not internal.is_selector_mode():
            return self.m
        selector_index = internal.build_selector_index()
        filtered_result = {}
        for k, v in self.m.items():
            if k in selector_index:
                select_data = internal.select_instance_attributes(
                    to_python_obj(self.m[k]), selector_index[k]
                )
                filtered_result[k] = to_kcl_obj(select_data) if to_kcl else select_data
        self.m = filtered_result or self.m
        return self.m


class KCLBytecode:
    def __init__(
        self,
        *,
        names: typing.List[str] = None,
        constants: typing.List[KCLObject] = None,
        instructions: typing.List[int] = None,
    ):
        self.names: typing.List[str] = names if names else []
        self.constants: typing.List[KCLObject] = constants if constants else []
        self.instructions: typing.List[int] = instructions if instructions else []


class KCLProgram:
    def __init__(
        self,
        *,
        root: str = "",
        main: str = "",
        pkgs: typing.Dict[str, KCLBytecode] = None,
    ):
        self.root: str = root if root else ""
        self.main: str = main if main else ""
        self.pkgs: typing.Dict[str, KCLBytecode] = pkgs if pkgs else {}
