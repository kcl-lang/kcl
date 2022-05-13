# Copyright 2021 The KCL Authors. All rights reserved.

import re
import pathlib
import pickle
import os
import hashlib
import time

from dataclasses import dataclass
from typing import Dict

import kclvm.api.version as ver_pkg
import kclvm.api.object as objpkg
import kclvm.kcl.ast as ast

from filelock import FileLock

# -------------
# Type alias
# -------------

CacheInfo = str
Cache = Dict[str, CacheInfo]

LOCK_SUFFIX = ".lock"
NORMAL_CACHE_SUFFIX = ".data"
BYTECODE_CACHE_PREFIX = "bytecode"
BYTECODE_CACHE_SUFFIX = ".kclc"
DEFAULT_CACHE_DIR = ".kclvm/cache"
FST_CACHE_DIR = ".kclvm/fst_cache"
CACHE_INFO_FILENAME = "info.pickle"


@dataclass
class CacheOption:
    cache_dir: str = DEFAULT_CACHE_DIR


def _get_cache_dir(root: str, cache_dir: str = DEFAULT_CACHE_DIR) -> str:
    return os.path.join(root, cache_dir, f"{ver_pkg.VERSION}-{ver_pkg.CHECKSUM}")


def _get_cache_filename(
    root: str, pkgpath: str, cache_dir: str = DEFAULT_CACHE_DIR
) -> str:
    return os.path.join(
        root,
        cache_dir,
        f"{ver_pkg.VERSION}-{ver_pkg.CHECKSUM}",
        f"{pkgpath}{NORMAL_CACHE_SUFFIX}",
    )


def _get_cache_info_filename(root: str, cache_dir: str = DEFAULT_CACHE_DIR):
    return os.path.join(
        root, cache_dir, f"{ver_pkg.VERSION}-{ver_pkg.CHECKSUM}", CACHE_INFO_FILENAME
    )


def _get_cache_bytecode_filename(
    root: str, check_sum: str, cache_dir: str = DEFAULT_CACHE_DIR
):
    return os.path.join(
        root,
        cache_dir,
        f"{ver_pkg.VERSION}-{ver_pkg.CHECKSUM}",
        f"{BYTECODE_CACHE_PREFIX}_{check_sum}{BYTECODE_CACHE_SUFFIX}",
    )


def read_info_cache(root: str, cache_dir: str = DEFAULT_CACHE_DIR) -> Cache:
    """Read the cache if it exists and is well formed.
    If it is not well formed, the call to write_info_cache later should resolve the issue.
    """
    cache_file = pathlib.Path(_get_cache_info_filename(root, cache_dir=cache_dir))
    if not cache_file.exists():
        return {}

    with cache_file.open("rb") as fobj:
        try:
            cache: Cache = pickle.load(fobj)
        except (pickle.UnpicklingError, ValueError):
            return {}

    return cache


def write_info_cache(
    cache: Cache, root: str, filepath: str, cache_dir: str = DEFAULT_CACHE_DIR
) -> None:
    """Update the cache info file."""
    dst_filename = _get_cache_info_filename(root, cache_dir=cache_dir)
    try:
        cache_dir = _get_cache_dir(root, cache_dir=cache_dir)
        pathlib.Path(cache_dir).mkdir(parents=True, exist_ok=True)
        relative_path = filepath.replace(root, ".", 1)
        new_cache = {
            **cache,
            **{relative_path: get_cache_info(filepath)},
        }
        tmp_filename = f"{cache_dir}/.{os.getpid()}{time.time_ns()}.tmp"
        # Write cache atomic
        with FileLock(dst_filename + LOCK_SUFFIX):
            f = open(tmp_filename, "wb")
            pickle.dump(new_cache, f)
            f.flush()
            os.fsync(f.fileno())
            f.close()
            os.rename(tmp_filename, dst_filename)
    except (pickle.UnpicklingError, ValueError):
        f.close()
        os.remove(tmp_filename)


def get_cache_info(path: str) -> CacheInfo:
    """Return the information used to check if a file or path is already changed or not."""
    path = pathlib.Path(path)
    check_sum = hashlib.md5()
    if os.path.isfile(path):
        with open(path, "rb") as f:
            check_sum.update(f.read())
    else:
        for file in list(sorted(path.glob("*.k"))):
            with open(file, "rb") as f:
                check_sum.update(f.read())
    return check_sum.hexdigest()


def get_pkg_realpath_from_pkgpath(root: str, pkgpath: str) -> str:
    """Get the pkgpath real path in the file system according to the root and pkgpath"""
    filepath = f"{root}/{pkgpath.replace('.', '/')}"
    if os.path.isfile(f"{filepath}.k"):
        filepath = f"{filepath}.k"
    return filepath


def load_data_from_file(filename) -> any:
    f = open(filename, "rb")
    # PyCharm
    # noinspection PyBroadException
    try:
        x = pickle.load(f)
        f.close()
        return x
    except Exception:
        f.close()
        os.remove(filename)
        return None


def save_data_to_file(dst_filename: str, tmp_filename: str, x: any):
    try:
        with FileLock(dst_filename + LOCK_SUFFIX):
            # write cache atomic
            f = open(tmp_filename, "wb")
            # PyCharm
            # noinspection PyBroadException
            pickle.dump(x, f)
            f.flush()
            os.fsync(f.fileno())
            f.close()
            os.rename(tmp_filename, dst_filename)
            return
    except Exception:
        f.close()
        os.remove(tmp_filename)
        return


def LoadPkgCache(root: str, pkgpath: str, option: CacheOption = CacheOption()) -> any:
    if not root or not pkgpath:
        return None

    filename = _get_cache_filename(root, pkgpath, cache_dir=option.cache_dir)
    if not os.path.exists(filename):
        return None

    # Compare the md5 using cache
    realpath = get_pkg_realpath_from_pkgpath(root, pkgpath)
    if os.path.exists(realpath):
        cache_info = read_info_cache(root, cache_dir=option.cache_dir)
        relative_path = realpath.replace(root, ".", 1)
        path_info_in_cache = cache_info.get(relative_path)
        path_info = get_cache_info(realpath)
        if path_info_in_cache != path_info:
            return None

    return load_data_from_file(filename)


def SavePkgCache(root: str, pkgpath: str, x: any, option: CacheOption = CacheOption()):
    if not root or not pkgpath or not x:
        return

    dst_filename = _get_cache_filename(root, pkgpath, cache_dir=option.cache_dir)

    # Save the pkgpath timesample and filesize into the cache
    realpath = get_pkg_realpath_from_pkgpath(root, pkgpath)
    if os.path.exists(realpath):
        cache_info = read_info_cache(root, cache_dir=option.cache_dir)
        write_info_cache(cache_info, root, realpath, cache_dir=option.cache_dir)

    cache_dir = _get_cache_dir(root, cache_dir=option.cache_dir)
    pathlib.Path(cache_dir).mkdir(parents=True, exist_ok=True)

    tmp_filename = f"{cache_dir}/{pkgpath}.{os.getpid()}{time.time_ns()}.tmp"

    save_data_to_file(dst_filename, tmp_filename, x)


def LoadMainPkgCache(
    root: str, filename: str, option: CacheOption = CacheOption()
) -> any:
    if not root or not filename:
        return None

    cache_name = filename.replace(root, "").replace("/", "_")
    cache_filename = _get_cache_filename(root, cache_name, cache_dir=option.cache_dir)

    if not os.path.exists(cache_filename):
        return None

    # Compare the md5 using cache
    if os.path.exists(filename):
        cache_info = read_info_cache(root, cache_dir=option.cache_dir)
        relative_path = filename.replace(root, ".", 1)
        path_info_in_cache = cache_info.get(relative_path)
        path_info = get_cache_info(filename)
        if path_info_in_cache != path_info:
            return None

    return load_data_from_file(cache_filename)


def SaveMainPkgCache(
    root: str, filename: str, x: any, option: CacheOption = CacheOption()
):
    if not root or not filename:
        return

    cache_name = filename.replace(root, "").replace("/", "_")
    dst_filename = _get_cache_filename(root, cache_name, cache_dir=option.cache_dir)

    if os.path.exists(filename):
        cache_info = read_info_cache(root, cache_dir=option.cache_dir)
        write_info_cache(cache_info, root, filename, cache_dir=option.cache_dir)

    cache_dir = _get_cache_dir(root, cache_dir=option.cache_dir)
    pathlib.Path(cache_dir).mkdir(parents=True, exist_ok=True)

    tmp_filename = f"{cache_dir}/{cache_name}.{os.getpid()}{time.time_ns()}.tmp"

    save_data_to_file(dst_filename, tmp_filename, x)


def LoadBytecodeCache(
    root: str, ast_program: ast.Program, option: CacheOption = CacheOption()
) -> objpkg.KCLProgram:
    if not root:
        return None
    if not ast_program or not isinstance(ast_program, ast.Program):
        return None
    if not ast_program.pkgs:
        return None
    check_sum = ast_program.get_check_sum(root)
    cache_filename = _get_cache_bytecode_filename(
        root, check_sum, cache_dir=option.cache_dir
    )
    if not os.path.exists(cache_filename):
        return None
    return load_data_from_file(cache_filename)


def SaveBytecodeCache(
    root: str,
    ast_program: ast.Program,
    program: objpkg.KCLProgram,
    option: CacheOption = CacheOption(),
):
    if not root:
        return
    if not ast_program or not isinstance(ast_program, ast.Program):
        return
    if not program or not isinstance(program, objpkg.KCLProgram):
        return
    pkgs = list(program.pkgs.keys()) if program.pkgs else None
    if not pkgs:
        return
    check_sum = ast_program.get_check_sum(root)
    dst_filename = _get_cache_bytecode_filename(
        root, check_sum, cache_dir=option.cache_dir
    )
    if os.path.exists(dst_filename):
        return
    cache_dir = _get_cache_dir(root, cache_dir=option.cache_dir)
    pathlib.Path(cache_dir).mkdir(parents=True, exist_ok=True)

    tmp_filename = f"{cache_dir}/{os.getpid()}{time.time_ns()}.kclc.tmp"

    save_data_to_file(dst_filename, tmp_filename, program)


def IsAbsPkgPath(s: str) -> bool:
    if not s or not isinstance(s, str):
        return False
    if s.startswith("."):
        return False
    if os.path.isabs(s):
        return False
    if ".." in s:
        return False
    if re.search(r"\s", s):
        return False

    return True


def IsRelPkgPath(s: str) -> bool:
    return s.strip().startswith(".") if s and isinstance(s, str) else False


def FixImportPath(root: str, filepath: str, import_path: str) -> str:
    """
    relpath: import .sub
    FixImportPath(root, "path/to/app/file.k", ".sub")        => path.to.app.sub
    FixImportPath(root, "path/to/app/file.k", "..sub")       => path.to.sub
    FixImportPath(root, "path/to/app/file.k", "...sub")      => path.sub
    FixImportPath(root, "path/to/app/file.k", "....sub")     => sub
    FixImportPath(root, "path/to/app/file.k", ".....sub")    => ""

    abspath: import path.to.sub
    FixImportPath(root, "path/to/app/file.k", "path.to.sub") => path.to.sub
    """
    assert root
    assert filepath
    assert import_path

    if not import_path.startswith("."):
        return import_path

    # Filepath to pkgpath
    pkgpath: str = (
        os.path.relpath(os.path.dirname(filepath), root).replace("/", ".").rstrip(".")
    )
    pkgpath = pkgpath.replace("\\", ".").rstrip(".")

    leading_dot_count = len(import_path)
    for i in range(len(import_path)):
        if import_path[i] != ".":
            leading_dot_count = i
            break

    # The pkgpath is the current root path
    if not pkgpath:
        return import_path.lstrip(".") if leading_dot_count <= 1 else ""

    if leading_dot_count == 1:
        return pkgpath + import_path

    ss = pkgpath.split(".")

    if (leading_dot_count - 1) < len(ss):
        return (
            ".".join(ss[: -(leading_dot_count - 1)])
            + "."
            + import_path[leading_dot_count:]
        )

    if (leading_dot_count - 1) == len(ss):
        return import_path[leading_dot_count:]

    return ""
