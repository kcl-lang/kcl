# Copyright 2021 The KCL Authors. All rights reserved.

import os
import typing
import sys
import subprocess
import glob
import platform

import kclvm.config
import kclvm.kcl.ast as ast
import kclvm.api.object as objpkg
import kclvm.compiler.build.compiler as compiler

from .native_runner import (
    LL_FILE_PATTERN,
    native_run_dylib,
)


def native_run_windows(
    path_list: typing.List[str], *, ast_prog: ast.Program
) -> objpkg.KCLResult:
    if platform.system() != "Windows":
        raise "native_run_windows only for windows"

    from kclvm.internal.kclx.transformer import transform_ast_to_kclx_ast_json_str

    # Config

    _no_link = True
    _executable_root = os.path.dirname(sys.executable)
    _kclvm_cli = f"{_executable_root}\\kclvm-cli.exe"
    _clang = f"{_executable_root}\\tools\\clang\\bin\\clang.exe"
    _clang = _clang if os.path.exists(_clang) else "clang.exe"

    _kclvm_main_win_c = f"{_executable_root}\\libs\\_kclvm_main_win.c"
    _kclvm_dll_lib = f"{_executable_root}\\libs\\kclvm.dll.lib"
    _kclvm_bc = f"{_executable_root}\\libs\\_kclvm.bc"
    _a_out_ast_json = "_a.out.ast.json"
    _a_out_ll = "_a.out.ll"
    _a_out_dylib = "_a.out.dll"
    _out_bc_files = []

    # Resolve Program
    compiler.CompileProgram(ast_prog)

    # Build Program with kclvm-cli, windows donot support cache

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
                "-Wno-override-module",
                "-shared",
                _a_out_ll,
                _kclvm_main_win_c,
                _kclvm_dll_lib,
                "-lws2_32",
                "-lbcrypt",
                "-lAdvapi32",
                "-lUserenv",
                "-o",
                _a_out_dylib,
            ]
            subprocess.check_call(args, stdout=open(os.devnull, "wb"))

        except subprocess.CalledProcessError as e:
            raise e

    # run app
    result = native_run_dylib(path_list, _a_out_dylib)
    if not kclvm.config.debug:
        if os.path.exists(_a_out_ast_json):
            os.remove(_a_out_ast_json)
        if _no_link:
            for file in _out_bc_files:
                if os.path.exists(file):
                    os.remove(file)
        else:
            if os.path.exists(_a_out_ll):
                os.remove(_a_out_ll)
        if os.path.exists(_a_out_dylib):
            os.remove(_a_out_dylib)

    return result


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

        for i in range(0, len(replaceArgs_old)):
            code = code.replace(replaceArgs_old[i], replaceArgs_new[i], -1)

        with open(dst_path, "w") as f:
            f.write(code)
