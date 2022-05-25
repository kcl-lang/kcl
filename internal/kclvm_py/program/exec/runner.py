# Copyright 2021 The KCL Authors. All rights reserved.

import os
import platform
import typing
import json

from ast import literal_eval

import kclvm.config
import kclvm.kcl.ast as ast
import kclvm.api.object as objpkg
import kclvm.tools.query as query
import kclvm.compiler.parser.parser as parser
import kclvm.compiler.build.compiler as compiler
import kclvm.compiler.vfs as vfs
import kclvm.vm as vm
import kclvm.internal.gpyrpc.gpyrpc_pb2 as pb2

from kclvm.api.object.internal import (
    kcl_option_reset,
    kcl_option_init_all,
)
from .native_runner import native_run, is_linux_platform
from .kclvm_cli import kclvm_cli_native_run_dylib


KCLVM_PANIC_INFO_KEY = "__kcl_PanicInfo__"
KCLVM_RUN_MODE_WITHIN_CACHE_ENV = "KCLVM_RUN_MODE_WITHIN_CACHE"
KCLVM_TARGET_ENV_KEY = "KCLVM_TARGET"


def Run(
    path_list: typing.List[str],
    *,
    work_dir: str = "",
    k_code_list: typing.List[str] = None,
    cmd_args: typing.List[ast.CmdArgSpec] = None,
    cmd_overrides: typing.List[ast.CmdOverrideSpec] = None,
    # -r --strict-range-check
    strict_range_check: bool = None,
    # -n --disable-none
    disable_none: bool = None,
    # -v --verbose
    verbose: int = None,
    # -d --debug
    debug: int = None,
    print_override_ast: bool = False,
    # --target
    target: str = "",
) -> objpkg.KCLResult:
    assert len(path_list) > 0

    if not work_dir and not k_code_list:
        for s in path_list:
            if os.path.isdir(s):
                work_dir = s
    if not work_dir and not k_code_list:
        work_dir = kclvm.config.current_path or os.path.dirname(path_list[0])

    root = vfs.MustGetPkgRoot(path_list)
    modfile = vfs.LoadModFile(root)
    target = (target or modfile.build.target or os.getenv(KCLVM_TARGET_ENV_KEY) or "").lower()

    kclvm.config.input_file = path_list
    kclvm.config.current_path = work_dir
    kclvm.config.is_target_native = target == "native"
    kclvm.config.is_target_wasm = target == "wasm"

    if strict_range_check is not None:
        kclvm.config.strict_range_check = strict_range_check
    if disable_none is not None:
        kclvm.config.disable_none = disable_none
    if verbose is not None:
        kclvm.config.verbose = verbose
    if debug is not None:
        kclvm.config.debug = debug

    if cmd_args:
        kclvm.config.arguments = []
        for x in cmd_args or []:
            try:
                better_value = literal_eval(x.value)
                kclvm.config.arguments.append((x.name, better_value))
            except Exception:
                kclvm.config.arguments.append((x.name, x.value))

    # rust: build/link/run
    if target == "native" or target == "wasm":
        kclvm.config.is_target_native = True

        args = pb2.ExecProgram_Args()
        args.work_dir = work_dir
        args.k_filename_list.extend(path_list)
        args.k_code_list.extend(k_code_list)

        for kv in kclvm.config.arguments or []:
            key, value = kv
            if isinstance(value, (bool, list, dict)):
                value = json.dumps(value)
            elif isinstance(value, str):
                value = '"{}"'.format(value.replace('"', '\\"'))
            else:
                value = str(value)
            args.args.append(
                pb2.CmdArgSpec(
                    name=key,
                    value=value,
                )
            )

        for x in cmd_overrides or []:
            args.overrides.append(
                pb2.CmdOverrideSpec(
                    pkgpath=x.pkgpath,
                    field_path=x.field_path,
                    field_value=x.field_value,
                    action=x.action.value,
                )
            )

        args.print_override_ast = print_override_ast or False
        args.strict_range_check = strict_range_check or False
        args.disable_none = disable_none or False
        args.verbose = verbose or 0
        args.debug = debug or 0

        return kclvm_cli_native_run_dylib(args)

    # Only for linux debug directly run
    from .native_runner import (
        get_path_list_dylib_path,
        native_run_dylib,
        native_try_run_dylib,
    )

    if (
        target == "native"
        and is_linux_platform()
        and not cmd_overrides
        and os.environ.get(KCLVM_RUN_MODE_WITHIN_CACHE_ENV)
    ):
        dylib_path = get_path_list_dylib_path(root, path_list)
        if os.path.exists(dylib_path):
            try:
                return native_run_dylib(path_list, dylib_path, should_exit=True)
            except Exception:
                result = native_try_run_dylib(root, path_list, dylib_path)
                if result:
                    return result

    ast_prog = parser.LoadProgram(
        *path_list,
        work_dir=work_dir,
        k_code_list=k_code_list,
        mode=parser.ParseMode.ParseComments if cmd_overrides else parser.ParseMode.Null,
    )
    ast_prog.cmd_args = cmd_args if cmd_args else []
    ast_prog.cmd_overrides = cmd_overrides if cmd_overrides else []

    # Apply argument
    kcl_option_reset()
    kcl_option_init_all()

    if target == "native":
        if platform.system() == "Windows":
            from .native_runner_windows_amd64 import native_run_windows

            result = native_run_windows(path_list, ast_prog=ast_prog)
        else:
            result = native_run(path_list, ast_prog=ast_prog)

    elif target == "wasm":
        from .native_runner_wasm32 import native_run_wasm32

        result = native_run_wasm32(path_list, ast_prog=ast_prog)

    else:
        # AST to bytecode list
        bin_prog = compiler.CompileProgram(
            ast_prog, enable_cache=not bool(ast_prog.cmd_overrides)
        )

        # Run bytecode list
        result = vm.Run(bin_prog)

    # If cmd overrides are used and config.debug is True, write back KCL files
    if print_override_ast:
        query.PrintOverridesAST()
    return result
