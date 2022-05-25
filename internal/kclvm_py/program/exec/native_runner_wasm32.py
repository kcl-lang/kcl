# Copyright 2021 The KCL Authors. All rights reserved.

import os
import typing
import sys
import subprocess
import glob
import platform
import json
import inspect

import kclvm.config
import kclvm.kcl.ast as ast
import kclvm.kcl.info as kcl_info
import kclvm.api.object as objpkg
import kclvm.compiler.build.compiler as compiler
import kclvm.compiler.extension.plugin.plugin as kcl_plugin
import kclvm.kcl.error as kcl_error

from .native_runner import (
    LL_FILE_PATTERN,
    KCLVM_PANIC_INFO_KEY,
)


def native_run_wasm32(
    path_list: typing.List[str], *, ast_prog: ast.Program
) -> objpkg.KCLResult:
    from kclvm.internal.kclx.transformer import transform_ast_to_kclx_ast_json_str

    # Config

    if platform.system() == "Windows":
        _no_link = True
        _executable_root = os.path.dirname(sys.executable)
        _kclvm_cli = f"{_executable_root}\\kclvm_cli.exe"
        _clang = f"{_executable_root}\\tools\\clang\\bin\\clang.exe"
        _clang = _clang if os.path.exists(_clang) else "clang.exe"

        _kclvm_bc = f"{_executable_root}\\include\\_kclvm.bc"
        _kclvm_lib_path = f"{_executable_root}\\libs"
        _kclvm_lib_name = "kclvm_wasm32"
        _kclvm_undefined_file = f"{_executable_root}\\libs\\_kclvm_undefined_wasm.txt"

    else:
        _no_link = True
        _executable_root = os.path.dirname(os.path.dirname(sys.executable))
        _kclvm_cli = f"{_executable_root}/bin/kclvm_cli"
        _clang = f"{_executable_root}/tools/clang/bin/clang"
        _clang = _clang if os.path.exists(_clang) else "clang"

        _kclvm_bc = f"{_executable_root}/include/_kclvm.bc"
        _kclvm_lib_path = f"{_executable_root}/lib"
        _kclvm_lib_name = "kclvm_wasm32"
        _kclvm_undefined_file = f"{_executable_root}/lib/_kclvm_undefined_wasm.txt"

    _a_out_ast_json = "_a.out.ast.json"
    _a_out_ll = "_a.out.ll"
    _a_out_wasm = "_a.out.wasm"
    _out_bc_files = []

    # Resolve Program
    compiler.CompileProgram(ast_prog)

    # Build Program with kclvm_cli, windows donot support cache

    if True:
        # Transfrom Program
        ast_json = transform_ast_to_kclx_ast_json_str(ast_prog)
        with open(_a_out_ast_json, "w") as file:
            file.write(ast_json)
        if _no_link and os.path.exists(_a_out_ll):
            os.remove(_a_out_ll)
        if _no_link:
            _out_bc_files = glob.glob(_a_out_ll + LL_FILE_PATTERN)
            for file in _out_bc_files:
                if os.path.exists(file):
                    os.remove(file)

        # kclvm compile
        try:
            args = [
                _kclvm_cli,
                "build",
                _a_out_ast_json,
                "--bc",
                _kclvm_bc,
                "-o",
                _a_out_ll + ".tmp.ll",
            ]
            subprocess.check_call(args)

            fix_ll_local_name(_a_out_ll, _a_out_ll + ".tmp.ll")
            os.remove(_a_out_ll + ".tmp.ll")

        except subprocess.CalledProcessError as e:
            raise e
        _out_bc_files = glob.glob(_a_out_ll + LL_FILE_PATTERN)

        # clang
        try:
            args = [
                _clang,
                "--target=wasm32-unknown-unknown-wasm",
                "-Wno-override-module",
                "-nostdlib",
                "-Wl,--no-entry",
                "-Wl,--export-all",
                f"-Wl,--allow-undefined-file={_kclvm_undefined_file}",
                "-O3",
                f"-L{_kclvm_lib_path}",
                f"-l{_kclvm_lib_name}",
                _a_out_ll,
                "-o",
                _a_out_wasm,
            ]

            if platform.system() == "Windows":
                subprocess.check_call(args, stdout=open(os.devnull, "wb"))
            else:
                subprocess.check_call(args)

        except subprocess.CalledProcessError as e:
            raise e

    # run wasm
    result = WasmApp(path_list, _a_out_wasm).run()
    if not kclvm.config.debug:
        if os.path.exists(_a_out_ast_json):
            os.remove(_a_out_ast_json)
        if os.path.exists(_a_out_ll):
            os.remove(_a_out_ll)
        if _no_link:
            for file in _out_bc_files:
                if os.path.exists(file):
                    os.remove(file)
        if os.path.exists(_a_out_wasm):
            os.remove(_a_out_wasm)

    return result


class WasmApp:
    def __init__(
        self, path_list: typing.List[str], wasm_path: str, should_exit: bool = False
    ):
        import wasmer
        import wasmer_compiler_cranelift

        self.path_list = path_list
        self.wasm_path = wasm_path
        self.should_exit = should_exit

        self.store = wasmer.Store(wasmer.engine.JIT(wasmer_compiler_cranelift.Compiler))
        self.module = wasmer.Module(self.store, open(wasm_path, "rb").read())

        def kclvm_plugin_invoke_json_wasm(method: int, args: int, kwargs: int) -> int:
            return self._invoke_json_wasm(method, args, kwargs)

        self.import_object = wasmer.ImportObject()
        self.import_object.register(
            "env",
            {
                "kclvm_plugin_invoke_json_wasm": wasmer.Function(
                    self.store, kclvm_plugin_invoke_json_wasm
                ),
            },
        )

        self.instance = wasmer.Instance(self.module, self.import_object)
        self.memory = self.instance.exports.memory

        self._invoke_json_buffer = self.instance.exports.kclvm_value_Str(0)
        self.instance.exports.kclvm_value_Str_resize(
            self._invoke_json_buffer, 1024 * 1024 * 1024
        )

    def _invoke_json_wasm(self, method_ptr: int, args_ptr: int, kwargs_ptr: int) -> int:
        method_len = self.instance.exports.kclvm_strlen(method_ptr)
        args_len = self.instance.exports.kclvm_strlen(args_ptr)
        kwargs_len = self.instance.exports.kclvm_strlen(kwargs_ptr)

        reader = bytearray(self.memory.buffer)

        method = reader[method_ptr : method_ptr + method_len].decode()
        args = reader[args_ptr : args_ptr + args_len].decode()
        kwargs = reader[kwargs_ptr : kwargs_ptr + kwargs_len].decode()

        json_result = self._call_py_method(method, args, kwargs)

        bytes_result = json_result.encode(encoding="utf8")
        self.instance.exports.kclvm_value_Str_resize(
            self._invoke_json_buffer, len(bytes_result) + 1
        )

        buf_ptr = self.instance.exports.kclvm_value_Str_ptr(self._invoke_json_buffer)
        buf_len = self.instance.exports.kclvm_value_Str_len(self._invoke_json_buffer)
        assert buf_len == len(bytes_result) + 1

        mem = self.memory.uint8_view()
        for i in range(len(bytes_result)):
            mem[buf_ptr + i] = bytes_result[i]
        mem[buf_ptr + len(bytes_result)] = 0

        return buf_ptr

    def run(self) -> objpkg.KCLResult:
        ctx = self.instance.exports.kclvm_context_new()
        result = self.instance.exports.kclvm_main(ctx)

        c_str_ptr = self.instance.exports.kclvm_value_Str_ptr(result)
        c_str_len = self.instance.exports.kclvm_value_len(result)

        reader = bytearray(self.memory.buffer)
        result = reader[c_str_ptr : c_str_ptr + c_str_len].decode()

        return self.json_to_object(result, None)

    def json_to_object(
        self, json_result: str, warn_json_result: str = None
    ) -> objpkg.KCLResult:
        if kclvm.config.list_option_mode:
            print(json_result, end="")
            return objpkg.KCLResult({})

        data = json.loads(json_result)

        panic_info = {}
        if KCLVM_PANIC_INFO_KEY in data:
            panic_info = data
        else:
            if warn_json_result:
                panic_info = json.loads(warn_json_result)
            else:
                panic_info = {}

        # check panic_info
        if panic_info.get(KCLVM_PANIC_INFO_KEY):
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

        return objpkg.KCLResult(data, os.path.abspath(self.path_list[-1]))

    def _call_py_method(self, name: str, args_json: str, kwargs_json: str) -> str:
        try:
            return self._call_py_method_unsafe(name, args_json, kwargs_json)
        except Exception as e:
            return json.dumps({"__kcl_PanicInfo__": f"{e}"})

    def _call_py_method_unsafe(
        self, name: str, args_json: str, kwargs_json: str
    ) -> str:
        dotIdx = name.rfind(".")
        if dotIdx < 0:
            return ""

        modulePath = name[:dotIdx]
        mathodName = name[dotIdx + 1 :]

        plugin_name = modulePath[modulePath.rfind(".") + 1 :]

        module = kcl_plugin.get_plugin(plugin_name)
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


def fix_ll_local_name(dst_path, src_path: str):
    replaceArgs_old = []
    replaceArgs_new = []

    for i in range(0, 10):
        replaceArgs_old.append(f"%{i}")
        replaceArgs_new.append(f"%local_{i}")

        replaceArgs_old.append(f"\n{i}")
        replaceArgs_new.append(f"\nlocal_{i}")

    with open(src_path, "r") as file:
        code = file.read()

        if platform.system() == "Windows":
            for i in range(0, len(replaceArgs_old)):
                code = code.replace(replaceArgs_old[i], replaceArgs_new[i], -1)

        with open(dst_path, "w") as f:
            f.write(code)
