# Copyright 2022 The KCL Authors. All rights reserved.

import os
import sys
import platform
import typing
import json
import inspect

from ctypes import *

import google.protobuf.json_format as json_format

import kclvm.compiler.extension.plugin.plugin as kcl_plugin
import kclvm.kcl.info as kcl_info
import kclvm.internal.gpyrpc.gpyrpc_pb2 as pb2
import kclvm.api.object as objpkg
import kclvm.kcl.error as kcl_error
import kclvm.config


kclvm_PANIC_INFO_KEY = "__kcl_PanicInfo__"

# Using kclvm rust cli PATH or current exec path.
_exe_root = os.path.dirname(os.environ.get("KCLVM_CLI_BIN_PATH") or sys.executable)
_cli_dll = None


def init_cli_dll():
    global _cli_dll

    if _cli_dll:
        return

    if platform.system() == "Darwin":
        _cli_dll_path = f"{_exe_root}/bin/libkclvm_cli_cdylib.dylib"
        _cli_dll = CDLL(_cli_dll_path)
    elif platform.system() == "Linux":
        _cli_dll_path = f"{_exe_root}/bin/libkclvm_cli_cdylib.so"
        _cli_dll = CDLL(_cli_dll_path)
    elif platform.system() == "Windows":
        _cli_dll_path = f"{_exe_root}/kclvm_cli_cdylib.dll"
        _cli_dll = CDLL(_cli_dll_path)
    else:
        raise f"unknown os: {platform.system()}"


class PluginContex:
    def __init__(self):
        self._plugin_dict: typing.Dict[str, any] = {}

    def call_method(self, name: str, args_json: str, kwargs_json: str) -> str:
        return self._call_py_method(name, args_json, kwargs_json)

    def _call_py_method(self, name: str, args_json: str, kwargs_json: str) -> str:
        try:
            return self._call_py_method_unsafe(name, args_json, kwargs_json)
        except Exception as e:
            return json.dumps({"__kcl_PanicInfo__": f"{e}"})

    def _get_plugin(self, plugin_name: str) -> typing.Optional[any]:
        if plugin_name in self._plugin_dict:
            return self._plugin_dict[plugin_name]

        module = kcl_plugin.get_plugin(plugin_name)
        self._plugin_dict[plugin_name] = module
        return module

    def _call_py_method_unsafe(
        self, name: str, args_json: str, kwargs_json: str
    ) -> str:
        dotIdx = name.rfind(".")
        if dotIdx < 0:
            return ""

        modulePath = name[:dotIdx]
        mathodName = name[dotIdx + 1 :]

        plugin_name = modulePath[modulePath.rfind(".") + 1 :]

        module = self._get_plugin(plugin_name)
        mathodFunc = None

        for func_name, func in inspect.getmembers(module):
            if func_name == kcl_info.demangle(mathodName):
                mathodFunc = func
                break

        args = []
        kwargs = {}

        if args_json:
            args = json.loads(args_json)
            if not isinstance(args, list):
                return ""
        if kwargs_json:
            kwargs = json.loads(kwargs_json)
            if not isinstance(kwargs, dict):
                return ""

        result = mathodFunc(*args, **kwargs)
        return json.dumps(result)


__plugin_context__ = PluginContex()
__plugin_method_agent_buffer__ = create_string_buffer(1024)


@CFUNCTYPE(c_char_p, c_char_p, c_char_p, c_char_p)
def plugin_method_agent(method: str, args_json: str, kwargs_json: str) -> c_char_p:
    method = str(method, encoding="utf-8")
    args_json = str(args_json, encoding="utf-8")
    kwargs_json = str(kwargs_json, encoding="utf-8")

    json_result = __plugin_context__.call_method(method, args_json, kwargs_json)

    global __plugin_method_agent_buffer__
    __plugin_method_agent_buffer__ = create_string_buffer(json_result.encode("utf-8"))
    return addressof(__plugin_method_agent_buffer__)


def kclvm_cli_run(args: pb2.ExecProgram_Args) -> str:
    init_cli_dll()

    _cli_dll.kclvm_cli_run.restype = c_char_p
    _cli_dll.kclvm_cli_run.argtypes = [c_char_p, c_void_p]

    args_json = json_format.MessageToJson(
        args, including_default_value_fields=True, preserving_proto_field_name=True
    )

    result_json = _cli_dll.kclvm_cli_run(args_json.encode("utf-8"), plugin_method_agent)
    return result_json.decode(encoding="utf-8")


def kclvm_cli_native_run_dylib(args: pb2.ExecProgram_Args) -> objpkg.KCLResult:
    json_result = kclvm_cli_run(args)
    warn_json_result = ""

    if json_result.startswith("ERROR:"):
        warn_json_result = json_result[len("ERROR:") :]
        json_result = "{}"

    try:
        data = json.loads(json_result)
    except Exception as e:
        raise Exception(f"Exception={e}, json_result={json_result}")

    panic_info = {}
    if kclvm_PANIC_INFO_KEY in data:
        panic_info = data
    else:
        if warn_json_result:
            try:
                panic_info = json.loads(warn_json_result)
            except Exception as e:
                raise Exception(f"Exception={e}, warn_json_result={warn_json_result}")
        else:
            panic_info = {}

    # check panic_info
    if panic_info.get(kclvm_PANIC_INFO_KEY):
        err_type_code = panic_info["err_type_code"]
        if err_type_code:
            err_type = kcl_error.ErrType((err_type_code,))
        else:
            err_type = kcl_error.ErrType.EvaluationError_TYPE

        file_msg = [
            kcl_error.ErrFileMsg(
                filename=panic_info.get("kcl_file"),
                line_no=panic_info.get("kcl_line"),
                col_no=panic_info.get("kcl_col"),
                arg_msg=panic_info.get("kcl_arg_msg"),
            )
        ]
        if kclvm.config.debug and kclvm.config.verbose >= 2:
            rust_filename = panic_info.get("rust_file")
            rust_line = panic_info.get("rust_line")
            rust_col = panic_info.get("rust_col")
            print(f"Rust error info: {rust_filename}:{rust_line}:{rust_col}")

        config_meta_file_msg = kcl_error.ErrFileMsg(
            filename=panic_info.get("kcl_config_meta_file"),
            line_no=panic_info.get("kcl_config_meta_line"),
            col_no=panic_info.get("kcl_config_meta_col"),
            arg_msg=panic_info.get("kcl_config_meta_arg_msg"),
        )
        if config_meta_file_msg.arg_msg:
            file_msg.append(config_meta_file_msg)

        if panic_info.get("is_warning") or panic_info.get("is_warnning"):
            kcl_error.report_warning(
                err_type=err_type, file_msgs=[], arg_msg=panic_info.get("message")
            )
        else:
            kcl_error.report_exception(
                err_type=err_type,
                file_msgs=file_msg,
                arg_msg=panic_info.get("message"),
            )

    return objpkg.KCLResult(data, os.path.abspath(args.k_filename_list[-1]))
