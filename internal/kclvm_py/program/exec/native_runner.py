# Copyright 2021 The KCL Authors. All rights reserved.

import os
import typing
import sys
import subprocess
import pathlib
import json
import glob
import hashlib
import platform
import shutil
import filelock

import kclvm.config
import kclvm.kcl.ast as ast
import kclvm.api.object as objpkg
import kclvm.compiler.build.compiler as compiler
import kclvm.compiler.vfs as vfs
import kclvm.kcl.error as kcl_error
import kclvm.api.version as ver_pkg


CLANG = "clang"
LL_FILE_PATTERN = "*.ll"
BC_FILE_PATTERN = "*.bc"
LL_FILE_SUFFIX = ".ll"
BC_FILE_SUFFIX = ".bc"
LOCK_FILE_SUFFIX = ".lock"
KCLVM_PANIC_INFO_KEY = "__kcl_PanicInfo__"
NATIVE_CACHE_DIR = ".kclvm/native_cache"
KCLVM_CLI_SUB_CMD = "build"
CACHE_OPTION = vfs.CacheOption(cache_dir=NATIVE_CACHE_DIR)


def native_run(
    path_list: typing.List[str], *, ast_prog: ast.Program
) -> objpkg.KCLResult:

    from kclvm.internal.kclx.transformer import transform_ast_to_kclx_ast_json_str

    # Config

    _no_link = True
    _exe = ".exe" if os.name == "nt" else ""
    _executable_root = os.path.dirname(os.path.dirname(sys.executable))
    _kclvm_cli = f"{_executable_root}/bin/kclvm-cli{_exe}"
    _clang = f"{_executable_root}/tools/clang/bin/{CLANG}{_exe}"
    _clang = _clang if os.path.exists(_clang) else f"{CLANG}{_exe}"
    _rust_libstd_name = (
        pathlib.Path(f"{_executable_root}/lib/rust-libstd-name.txt").read_text().strip()
    )
    _rust_libstd_dylib = f"{_executable_root}/lib/{_rust_libstd_name}"
    _kclvm_bc = f"{_executable_root}/include/_kclvm.bc"
    _a_out_ast_json = "_a.out.ast.json"
    _a_out_bc = "_a.out.ll"
    _lib_suffix = get_dylib_suffix()
    _a_out_dylib = f"_a.out.{_lib_suffix}"
    _out_bc_files = []

    # Resolve Program
    compiler.CompileProgram(ast_prog)
    # Build Program with kclvm-cli, clang with cache
    root = ast_prog.root
    modfile = vfs.LoadModFile(root)
    enable_cache = modfile.build.enable_pkg_cache
    if enable_cache:
        build_paths = []
        check_sum = ast_prog.get_check_sum(root)
        cache_dir = f"{root}/{NATIVE_CACHE_DIR}/{ver_pkg.VERSION}-{ver_pkg.CHECKSUM}"
        pathlib.Path(cache_dir).mkdir(parents=True, exist_ok=True)
        for pkgpath in ast_prog.pkgs:
            is_main_pkg = pkgpath == ast.Program.MAIN_PKGPATH
            compile_prog = ast.Program(
                root=root, main=ast_prog.main, pkgs={pkgpath: ast_prog.pkgs[pkgpath]}
            )
            if is_main_pkg:
                check_sum = compile_prog.get_check_sum(root)
                main_virtual_filename = f"{check_sum}.k"
            else:
                main_virtual_filename = ""
            bc_path = (
                f"{cache_dir}/{pkgpath}"
                if not is_main_pkg
                else f"{cache_dir}/{check_sum}"
            )
            with filelock.FileLock(bc_path + LOCK_FILE_SUFFIX):
                dylib_relative_path = (
                    vfs.LoadMainPkgCache(
                        root, main_virtual_filename, option=CACHE_OPTION
                    )
                    if is_main_pkg
                    else vfs.LoadPkgCache(root, pkgpath, option=CACHE_OPTION)
                )
                # If AST module has been modified, ignore the dylib cache
                if ast_prog.cmd_overrides and is_main_pkg:
                    dylib_relative_path = None
                if dylib_relative_path is None:
                    # Build dylib
                    ast_json = transform_ast_to_kclx_ast_json_str(compile_prog)
                    with open(_a_out_ast_json, "w") as file:
                        file.write(ast_json)
                    dylib_path = (
                        f"{cache_dir}/{pkgpath}.{_lib_suffix}"
                        if not is_main_pkg
                        else f"{cache_dir}/{check_sum}.{_lib_suffix}"
                    )
                    if kclvm.config.verbose > 3:
                        print(f"Compiling {pkgpath}")
                    if os.path.exists(bc_path):
                        os.remove(bc_path)
                    process = subprocess.run(
                        [
                            _kclvm_cli,
                            KCLVM_CLI_SUB_CMD,
                            _a_out_ast_json,
                            "--bc",
                            _kclvm_bc,
                            "-o",
                            bc_path,
                            "--linkmode",
                            "no_link",
                        ]
                    )
                    if process.returncode != 0:
                        raise Exception(
                            f"stdout: {process.stdout}, stderr: {process.stderr}"
                        )
                    process = subprocess.run(
                        [
                            _clang,
                            "-Wno-override-module",
                            "-Wno-error=unused-command-line-argument",
                            "-Wno-unused-command-line-argument",
                            "-shared",
                            "-undefined",
                            "dynamic_lookup",
                            f"-Wl,-rpath,{_executable_root}/lib",
                            f"-L{_executable_root}/lib",
                            "-lkclvm_native_shared",
                            f"-I{_executable_root}/include",
                            bc_path + LL_FILE_SUFFIX,
                            _rust_libstd_dylib,
                            "-fPIC",
                            "-o",
                            dylib_path,
                        ]
                    )
                    if process.returncode != 0:
                        raise Exception(
                            f"stdout: {process.stdout}, stderr: {process.stderr}"
                        )
                    dylib_relative_path = dylib_path.replace(root, ".", 1)
                    if not is_main_pkg:
                        vfs.SavePkgCache(
                            root, pkgpath, dylib_relative_path, option=CACHE_OPTION
                        )
                    else:
                        vfs.SaveMainPkgCache(
                            root,
                            main_virtual_filename,
                            dylib_relative_path,
                            option=CACHE_OPTION,
                        )
                    _out_bc_files.append(bc_path + LL_FILE_SUFFIX)
                else:
                    if dylib_relative_path.startswith("."):
                        dylib_path = dylib_relative_path.replace(".", root, 1)
            build_paths.append(dylib_path)
        _a_out_dylib = f"{cache_dir}/{check_sum}.out.{_lib_suffix}"

        process = subprocess.run(
            [
                _clang,
                "-Wno-override-module",
                "-Wno-error=unused-command-line-argument",
                "-Wno-unused-command-line-argument",
                "-shared",
                "-undefined",
                "dynamic_lookup",
                f"-Wl,-rpath,{_executable_root}/lib",
                f"-L{_executable_root}/lib",
                "-lkclvm_native_shared",
                f"-I{_executable_root}/include",
                *build_paths,
                _rust_libstd_dylib,
                "-fPIC",
                "-o",
                _a_out_dylib,
            ]
        )
        if process.returncode != 0:
            raise Exception(f"stdout: {process.stdout}, stderr: {process.stderr}")
    else:
        # Transfrom Program
        ast_json = transform_ast_to_kclx_ast_json_str(ast_prog)
        with open(_a_out_ast_json, "w") as file:
            file.write(ast_json)
        if _no_link and os.path.exists(_a_out_bc):
            os.remove(_a_out_bc)
        if _no_link:
            _out_bc_files = glob.glob(_a_out_bc + LL_FILE_PATTERN)
            for file in _out_bc_files:
                if os.path.exists(file):
                    os.remove(file)
        # kclvm compile
        process = subprocess.run(
            [
                _kclvm_cli,
                KCLVM_CLI_SUB_CMD,
                _a_out_ast_json,
                "--bc",
                _kclvm_bc,
                "-o",
                _a_out_bc,
                *(["--linkmode", "no_link"] if _no_link else []),
            ]
        )
        if process.returncode != 0:
            raise Exception(f"stdout: {process.stdout}, stderr: {process.stderr}")
        _out_bc_files = glob.glob(_a_out_bc + LL_FILE_PATTERN)
        # clang
        if _no_link:
            process = subprocess.run(
                [
                    _clang,
                    "-Wno-override-module",
                    "-Wno-error=unused-command-line-argument",
                    "-Wno-unused-command-line-argument",
                    "-shared",
                    "-undefined",
                    "dynamic_lookup",
                    f"-Wl,-rpath,{_executable_root}/lib",
                    f"-L{_executable_root}/lib",
                    "-lkclvm_native",
                    f"-I{_executable_root}/include",
                    *_out_bc_files,
                    _rust_libstd_dylib,
                    f"{_executable_root}/lib/libkclvm.{_lib_suffix}",
                    "-fno-lto",
                    "-fPIC",
                    "-o",
                    _a_out_dylib,
                ]
            )
        else:
            process = subprocess.run(
                [
                    _clang,
                    "-Wno-override-module",
                    "-Wno-error=unused-command-line-argument",
                    "-Wno-unused-command-line-argument",
                    "-shared",
                    "-undefined",
                    "dynamic_lookup",
                    f"-Wl,-rpath,{_executable_root}/lib",
                    f"-L{_executable_root}/lib",
                    "-lkclvm_native",
                    f"-I{_executable_root}/include",
                    _a_out_bc,
                    _rust_libstd_dylib,
                    f"{_executable_root}/lib/libkclvm.{_lib_suffix}",
                    "-fno-lto",
                    "-fPIC",
                    "-o",
                    _a_out_dylib,
                ]
            )
        if process.returncode != 0:
            raise Exception(f"stdout: {process.stdout}, stderr: {process.stderr}")

    # run app
    result = native_run_dylib(path_list, _a_out_dylib)

    if not kclvm.config.debug:
        if os.path.exists(_a_out_ast_json):
            os.remove(_a_out_ast_json)
        if not enable_cache:
            if _no_link:
                for file in _out_bc_files:
                    if os.path.exists(file):
                        os.remove(file)
            else:
                if os.path.exists(_a_out_bc):
                    os.remove(_a_out_bc)
            if os.path.exists(_a_out_dylib):
                os.remove(_a_out_dylib)

    return result


def native_run_dylib(
    path_list: typing.List[str], dylib_path: str, should_exit: bool = False
) -> objpkg.KCLResult:
    """Native run with dylib"""
    import kclvm_plugin as kclvm_plugin

    # run app
    ctx = kclvm_plugin.AppContext(os.path.abspath(dylib_path))

    # init options
    ctx.InitOptions(kclvm.config.arguments)

    # run app
    json_result = ctx.RunApp(
        strict_range_check=kclvm.config.strict_range_check,
        disable_none=kclvm.config.disable_none,
        disable_schema_check=kclvm.config.disable_schema_check,
        list_option_mode=kclvm.config.list_option_mode,
        debug_mode=kclvm.config.debug,
    )
    warn_json_result = ctx.GetWarn()

    if kclvm.config.list_option_mode:
        print(json_result, end="")
        return objpkg.KCLResult({})

    try:
        data = json.loads(json_result)
    except Exception as e:
        raise Exception(f"Exception={e}, json_result={json_result}")

    panic_info = {}
    if KCLVM_PANIC_INFO_KEY in data:
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

    return objpkg.KCLResult(data, os.path.abspath(path_list[-1]))


def native_try_run_dylib(
    root, path_list: typing.List[str], dylib_path: str
) -> typing.Optional[objpkg.KCLResult]:
    if os.path.exists(dylib_path):
        if is_linux_platform():
            # Run ldd
            _ldd = "ldd"
            _so_not_found_flag = " => not found"
            try:
                process = subprocess.run(
                    [
                        _ldd,
                        dylib_path,
                    ],
                    capture_output=True,
                )
                if process.returncode != 0:
                    return None
                linked_so_path_list = [
                    p.strip("\t ")
                    for p in str(process.stdout, encoding="utf-8").split("\n")
                    if p
                ]
                not_found_so_path_list = [
                    p.replace(_so_not_found_flag, "").strip()
                    for p in linked_so_path_list
                    if _so_not_found_flag in p
                ]
                target_path = not_found_so_path_list[-1].rsplit("/", 1)[0]
                source_path = (
                    f"{root}/{NATIVE_CACHE_DIR}/{ver_pkg.VERSION}-{ver_pkg.CHECKSUM}"
                )

                if os.path.exists(target_path):
                    shutil.rmtree(target_path)
                os.makedirs(target_path)
                if os.path.exists(source_path):
                    for root, dirs, files in os.walk(source_path):
                        for file in files:
                            src_file = os.path.join(root, file)
                            shutil.copy(src_file, target_path)

                return native_run_dylib(path_list, dylib_path, should_exit=True)
            except Exception:
                return None
    return None


def get_path_list_dylib_path(root: str, path_list: typing.List[str]) -> str:
    check_sum = hashlib.md5()
    if not path_list or not root:
        return ""
    for filename in path_list:
        if os.path.isfile(filename):
            filename = os.path.abspath(filename)
            check_sum.update(
                (filename.replace(root, ".", 1) if root else filename).encode(
                    encoding="utf-8"
                )
            )
            with open(filename, "rb") as f:
                check_sum.update(f.read())
    cache_dir = f"{root}/{NATIVE_CACHE_DIR}/{ver_pkg.VERSION}-{ver_pkg.CHECKSUM}"
    dylib_path = "{}/{}.out.{}".format(
        cache_dir, check_sum.hexdigest(), get_dylib_suffix()
    )
    return dylib_path


def get_dylib_suffix() -> str:
    """Get dylib suffix on diffrent platform"""
    sysstr = platform.system()
    if sysstr == "Windows":
        lib_suffix = "dll"
    elif sysstr == "Linux":
        lib_suffix = "so"
    else:
        lib_suffix = "dylib"
    return lib_suffix


def is_linux_platform() -> bool:
    """Platform is linux"""
    return platform.system() == "Linux"
