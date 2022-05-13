# Copyright 2021 The KCL Authors. All rights reserved.

import typing
import pathlib
import os
import sys

import toml
import google.protobuf.json_format as json_format

import kclvm.config
import kclvm.kcl.error as kcl_error


def GetPkgRoot(
    k_file_path: str, should_current_file_work_dir: bool = True
) -> typing.Optional[str]:
    """Search kcl.mod filepath with the KCL file path"""
    if not k_file_path:
        return None

    # search by kcl.mod file
    module_path = pathlib.Path(os.path.abspath(k_file_path))
    root = module_path.root
    while module_path:
        if module_path == module_path.parent or str(module_path) == root:
            break

        kcl_mod_path = module_path.joinpath("kcl.mod")
        if kcl_mod_path.exists() and kcl_mod_path.is_file():
            return str(module_path)

        module_path = module_path.parent

    if should_current_file_work_dir and k_file_path.endswith(".k"):
        return os.path.dirname(k_file_path)

    return None


def MustGetPkgRoot(file_paths: typing.List[str]) -> typing.Optional[str]:
    """Search kcl.mod filepath with the KCL file paths,
    when found multiple kcl.mod paths, raise a compile error.
    """
    # Get kcl.mod paths from input file paths and remove empty path using the filter function.
    paths = set(
        filter(
            None,
            [
                GetPkgRoot(file_path, should_current_file_work_dir=False)
                for file_path in file_paths or []
            ],
        )
    )
    # Not found kcl.mod.
    if not paths:
        return None
    # Find one kcl.mod.
    if len(paths) == 1:
        return list(paths)[0]
    # Find multiple kcl.mod, raise an error.
    kcl_error.report_exception(
        err_type=kcl_error.ErrType.CompileError_TYPE,
        arg_msg=f"conflict kcl.mod file paths: {paths}",
    )


def LoadModFile(root: str) -> kclvm.config.KclModFile:
    k_mod_file_path = f"{root}/kcl.mod"

    if not os.path.exists(k_mod_file_path):
        mod_file = kclvm.config.KclModFile(root=root)
        return mod_file

    d = toml.load(k_mod_file_path)
    mod_file = kclvm.config.KclModFile(root=root)
    json_format.ParseDict(d, mod_file, ignore_unknown_fields=True)
    mod_file.root = root
    return mod_file


if __name__ == "__main__":
    if len(sys.argv) < 2 or (sys.argv[1] == "-h" or sys.argv[1] == "-help"):
        print("usage: python3 ./this_py_file <kcl.mod root>")
        sys.exit(0)

    f = LoadModFile(sys.argv[1])
    print(f)
