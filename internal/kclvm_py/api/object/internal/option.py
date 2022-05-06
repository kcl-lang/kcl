# Copyright 2021 The KCL Authors. All rights reserved.

import typing
import ast
import os

import kclvm.kcl.error as kcl_error
import kclvm.internal.util as util
import kclvm.config


class KclOptionElem:
    def __init__(self, name: str):
        self.name = name
        self.defined = False
        self.value_type = ""  # bool/int/float/str/list/dict
        self.required = False
        self.default = None
        self.help = ""
        self.file = ""
        self.line = 0

        self.inited = False
        self.value = None

    def get_help(self, verbose_mode=0) -> str:
        name = self.name
        default = self.default if self.default else "?"

        type_and_required = ""
        if self.value_type != "" and self.required:
            type_and_required = f" ({self.value_type},required)"
        elif self.value_type != "":
            type_and_required = f" ({self.value_type})"
        elif self.required:
            type_and_required = " (required)"

        if verbose_mode > 1 and self.file and self.line > 0:
            filename = os.path.relpath(self.file, os.getcwd())
            return f"{name}={default}{type_and_required} {self.help} ({filename}:{self.line})".strip()
        else:
            return f"{name}={default}{type_and_required} {self.help}".strip()

    def __str__(self) -> str:
        return self.get_help()


class _KclOptionDict:
    def __init__(self):
        self.m: typing.Dict[str, KclOptionElem] = {}  # map[name]KclOptionElem
        self.reset()

    def reset(self):
        self.m = {}  # map[name]KclOptionElem

    def len(self) -> int:
        return len(self.m)

    def get_dict(self):
        return self.m  # map[name]KclOptionElem

    def keys(self) -> list:
        return list(self.m.keys())

    def has_key(self, name: str) -> bool:
        return name in self.m

    @classmethod
    def _check_value_type(cls, value_type: str, value: typing.Any) -> bool:
        if not value_type or value is None:
            return True
        return cls._get_typed_value(value_type, value) is not None

    # noinspection PyBroadException
    @classmethod
    def _get_typed_value(cls, value_type: str, value) -> typing.Any:
        if not value_type:
            return value
        if value is None:
            return None

        if value_type == "bool":
            return True if str(value).lower() in ("yes", "true", "1") else bool(value)
        if value_type == "int":
            try:
                return int(value)
            except Exception:
                return None
        if value_type == "float":
            try:
                return float(value)
            except Exception:
                return None
        if value_type == "str":
            return str(value)
        if value_type == "list":
            if isinstance(value, list):
                return value
            try:
                result = ast.literal_eval(value)
                return result if type(result) is list else None
            except Exception:
                return None
        if value_type == "dict":
            if isinstance(value, dict):
                return value
            try:
                result = ast.literal_eval(value)
                return result if type(result) is dict else None
            except Exception:
                return None

        # unknown type
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.EvaluationError_TYPE,
            arg_msg=f"unknown type: {value_type}",
        )

    def option(
        self,
        name: str,
        *,
        value_type="",
        required=False,
        default=None,
        help="",
        file="",
        line=0,
    ) -> typing.Any:

        if value_type and value_type not in [
            "bool",
            "int",
            "float",
            "str",
            "list",
            "dict",
        ]:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.EvaluationError_TYPE,
                arg_msg=f"'{value_type}' must one of bool/int/float/str/list/dict or empty string",
            )

        opt = self.get(name)
        opt.defined = True

        if value_type:
            opt.value_type = value_type
        if required:
            opt.required = required
        if default is not None:
            opt.default = default
        if help:
            opt.help = help
        if file:
            opt.file = file
        if line > 0:
            opt.line = line

        if opt.value is None:
            opt.value = opt.default

        if value_type and opt.value is not None:
            raw_value = opt.value
            opt.value = self._get_typed_value(value_type, opt.value)
            if opt.value is None:
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.EvaluationError_TYPE,
                    arg_msg=f"cannot use '{raw_value}' as type '{value_type}'",
                )

        self.m[name] = opt
        kcl_option_exec()  # FIXME: filename, lineno, colno info
        return opt.value  # Do type conversion?

    def init_value(self, *d_list, name: str = "", value=None, **attrs):
        def _init_kv(name_: str, value_: str):
            opt = self.get(name_)
            opt.value = value_
            opt.inited = True
            self.m[name_] = opt

        for d in d_list:
            kv = d.split("=")
            if len(kv) != 2 or not kv[0]:
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.EvaluationError_TYPE,
                    arg_msg=f"invalid args: {d}",
                )
            _init_kv(kv[0], kv[1])

        for name, value in attrs.items():
            _init_kv(name, value)

        if name:
            _init_kv(name, value)

    def get(self, name: str) -> KclOptionElem:
        if name in self.m:
            return self.m[name]
        else:
            return KclOptionElem(name)

    def check(self, *, check_required=True, check_none=False, verbose_mode=0):
        for name, v in self.m.items():
            if verbose_mode > 1 and not v.defined:
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.EvaluationError_TYPE,
                    file_msgs=[kcl_error.ErrFileMsg(line_no=1)],
                    arg_msg=f"invalid '-D {name}={v.value}', option('{name}') undefined",
                )
            if check_required and v.required and (not v.inited):
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.EvaluationError_TYPE,
                    file_msgs=[kcl_error.ErrFileMsg(filename=v.file, line_no=v.line)],
                    arg_msg=f"option('{name}') must be initialized, try '-D {name}=?' argument",
                )
            if check_none and not v.value:
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.EvaluationError_TYPE,
                    file_msgs=[kcl_error.ErrFileMsg(filename=v.file, line_no=v.line)],
                    arg_msg=f"option('{name}') is None",
                )
            if not self._check_value_type(v.value_type, v.value):
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.EvaluationError_TYPE,
                    file_msgs=[kcl_error.ErrFileMsg(line_no=1)],
                    arg_msg=f"cannot use '-D {name}={v.value}' as type {v.value_type}",
                )

        return None

    def help(self, *, prefix: str = "", verbose_mode=0) -> str:
        if len(self.m) == 0:
            return ""

        msg = "option list:\n"
        for name in self.keys():
            msg += f"{prefix}{self.get(name).get_help(verbose_mode)}\n"
        return msg[:-1]  # remove \n


_kcl_option_dict = _KclOptionDict()


def kcl_option_dict():
    return _kcl_option_dict.get_dict()


def kcl_option_help(verbose_mode=0) -> str:
    return _kcl_option_dict.help(prefix="  -D ", verbose_mode=verbose_mode)


def kcl_option_check(verbose_mode=0):
    _kcl_option_dict.check(verbose_mode=verbose_mode)


def kcl_option_init(*args, name: str = "", value=None, **attrs):
    _kcl_option_dict.init_value(*args, name=name, value=value, **attrs)


def kcl_option(
    name: str, *, type="", required=False, default=None, help="", file="", line=0
) -> typing.Any:
    return _kcl_option_dict.option(
        name,
        value_type=type,
        required=required,
        default=default,
        help=help,
        file=file,
        line=line,
    )


def kcl_option_reset():
    _kcl_option_dict.reset()


def kcl_option_exec():
    if kclvm.config.list_option_mode > 0:
        kclvm.config.options_help_message = kcl_option_help(
            verbose_mode=kclvm.config.list_option_mode
        )
    else:
        kcl_option_check()


def kcl_option_init_all():
    if kclvm.config.arguments:
        for _k, _v in util.merge_option_same_keys(kclvm.config.arguments).items():
            kcl_option_init(name=_k, value=_v)
