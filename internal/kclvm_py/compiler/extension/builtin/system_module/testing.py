# Copyright 2020 The KCL Authors. All rights reserved.

import os
import typing


def KMANGLED_arguments(name: str, value: typing.Union[bool, int, float, str]) -> None:
    """Set arguments for option function in test."""

    assert isinstance(name, str)
    assert isinstance(value, (bool, int, float, str))

    # TODO: KMANGLED_arguments Support complex parameter types

    assert name, f"testing.arguments: '{name}' is invalid name"
    return


def KMANGLED_setting_file(filename: str) -> None:
    """Set setting file for option function in test."""
    assert isinstance(filename, str)

    assert os.path.exists(filename), f"testing.setting_file: '{filename}' not exists"
    assert os.path.isfile(filename), f"testing.setting_file: '{filename}' is not file"
    return
