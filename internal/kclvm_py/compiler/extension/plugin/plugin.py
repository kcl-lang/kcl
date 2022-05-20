# Copyright 2020 The KCL Authors. All rights reserved.

import os
import glob
import re
import copy
import pathlib
import typing
import sys
import traceback
import inspect
import platform

import kclvm.kcl.info as kcl_info
import kclvm.compiler.extension.plugin.template as plugin_template


UNKNOWN_VERSION = "unknown"


class Init:
    def __init__(self, root: str = "", info_map=None):
        if info_map is None:
            info_map = {}
        self.plugins_root = root or ""
        self.plugins_info_map = info_map or {}


_re_plugin_name = re.compile("^[a-z][a-z0-9_-]*$")


def _is_valid_plugin_name(plugin_name: str) -> bool:
    return _re_plugin_name.match(plugin_name) is not None


def _normalize_plugin_name(plugin_name: str) -> str:
    if plugin_name.startswith("kcl_plugin.") or plugin_name.startswith("kcl_plugin/"):
        plugin_name = plugin_name[len("kcl_plugin.") :]
    return plugin_name


def _get_plugin(root: str, plugin_name: str) -> typing.Optional[any]:
    if not os.path.exists(f"{root}/{plugin_name}/plugin.py"):
        return None

    import importlib.util

    spec = importlib.util.spec_from_file_location(
        f"kcl_plugin.{plugin_name}", f"{root}/{plugin_name}/plugin.py"
    )
    pkg = importlib.util.module_from_spec(spec)

    try:
        spec.loader.exec_module(pkg)
    except Exception:
        ex_type, ex_val, ex_stack = sys.exc_info()
        print(
            f"WARN: {root}/{plugin_name}/plugin.py:{traceback.extract_tb(ex_stack)[-1].lineno}: init_plugin failed: {ex_val}"
        )
        return None

    func_map = {}
    for func_name, func_body in inspect.getmembers(pkg, inspect.isfunction):
        if func_name.startswith(kcl_info.MANGLE_PREFIX):
            func_map[func_name[len(kcl_info.MANGLE_PREFIX) :]] = func_body

    for func_name, func_body in inspect.getmembers(pkg, inspect.isfunction):
        if not func_name.startswith(kcl_info.MANGLE_PREFIX) and func_name[0] != "_":
            if func_name not in func_map:
                func_map[kcl_info.MANGLE_PREFIX + func_name] = func_body

    for func_name in func_map:
        setattr(pkg, func_name, func_map[func_name])

    return pkg


def _get_info(pkg) -> dict:
    info = copy.deepcopy(getattr(pkg, "INFO"))

    info["method"] = {}
    for s in dir(pkg):
        if s.startswith(kcl_info.MANGLE_PREFIX):
            func_name = s[len(kcl_info.MANGLE_PREFIX) :]
            func_doc = getattr(pkg, s).__doc__
            info["method"][func_name] = func_doc if func_doc else "no doc"

    return info


def find_plugin_root() -> typing.Optional[str]:
    return _find_plugin_root()


def _find_plugin_root() -> typing.Optional[str]:
    # 1. try $KCL_PLUGINS_ROOT env
    env_plugin_root = os.getenv("KCL_PLUGINS_ROOT", "")
    if env_plugin_root != "":
        return env_plugin_root

    # 2. try ${pwd}/.../plugins/hello/plugin.py
    cwd_plugin_path = pathlib.Path(os.getcwd()).absolute()
    root = cwd_plugin_path.root
    while cwd_plugin_path:
        if cwd_plugin_path == cwd_plugin_path.parent or str(cwd_plugin_path) == root:
            break
        plugin_list_file_path = cwd_plugin_path.joinpath("plugins/hello/plugin.py")
        if plugin_list_file_path.exists() and plugin_list_file_path.is_file():
            return str(cwd_plugin_path.joinpath("plugins"))
        if cwd_plugin_path.joinpath("kcl.mod").exists():
            break
        cwd_plugin_path = cwd_plugin_path.parent

    # 3. try ${__file__}/.../plugins/hello/plugin.py
    cwd_plugin_path = pathlib.Path(__file__).parent.absolute()
    root = cwd_plugin_path.root
    while cwd_plugin_path:
        if cwd_plugin_path == cwd_plugin_path.parent or str(cwd_plugin_path) == root:
            break
        plugin_list_file_path = cwd_plugin_path.joinpath("plugins/hello/plugin.py")
        if plugin_list_file_path.exists() and plugin_list_file_path.is_file():
            return str(cwd_plugin_path.joinpath("plugins"))
        cwd_plugin_path = cwd_plugin_path.parent

    # 4. try $HOME/.kusion/kclvm/plugins
    home_dir = os.getenv("HOME") if platform.system() != "Windows" else os.getenv("UserProfile")
    home_plugin_root = os.path.join(home_dir, ".kusion/kclvm/plugins")
    if os.path.exists(f"{home_plugin_root}/hello/plugin.py"):
        return home_plugin_root

    # 5. not found
    return None


def _init_plugin_root() -> typing.Tuple[typing.Optional[str], dict]:
    plugins_root = _find_plugin_root()
    if plugins_root is None:
        return None, {}

    plugins_info = {}

    # 'hello' is builtin plugin, and used in test code
    if not os.path.exists(f"{plugins_root}/hello/plugin.py"):
        os.makedirs(f"{plugins_root}/hello")

        with open(f"{plugins_root}/hello/plugin.py", "w") as file:
            file.write(plugin_template.get_plugin_template_code("hello"))
        with open(f"{plugins_root}/hello/plugin_test.py", "w") as file:
            file.write(plugin_template.get_plugin_test_template_code("hello"))

    # scan all plugins
    k_files = glob.glob(f"{plugins_root}/*/plugin.py", recursive=False)
    for i in range(len(k_files)):
        plugin_name = os.path.basename(k_files[i][: -len("/plugin.py")])
        if _is_valid_plugin_name(plugin_name):
            pkg = _get_plugin(plugins_root, plugin_name)
            if not pkg:
                continue
            info = _get_info(pkg)

            plugins_info[plugin_name] = info
    return plugins_root, plugins_info


# init plugins
_plugin_root, _plugin_info_map = _init_plugin_root()
init_ = Init(_plugin_root, _plugin_info_map)


# -----------------------------------------------------------------------------
# API
# -----------------------------------------------------------------------------


def reset_plugin(plugin_root: str = ""):
    global _plugin_root
    global _plugin_info_map
    global init_

    os.environ["KCL_PLUGINS_ROOT"] = f"{plugin_root}"
    _plugin_root, _plugin_info_map = _init_plugin_root()
    init_ = Init(_plugin_root, _plugin_info_map)


def get_plugin_version() -> str:
    if not init_.plugins_root:
        return UNKNOWN_VERSION
    version_path = pathlib.Path(f"{init_.plugins_root}/VERSION")
    if version_path.exists():
        return version_path.read_text()
    return UNKNOWN_VERSION


def get_plugin_root(plugin_name: str = "") -> typing.Optional[str]:
    if init_.plugins_root is None:
        return None
    if plugin_name != "":
        plugin_name = _normalize_plugin_name(plugin_name)
        return f"{init_.plugins_root}/{plugin_name}"

    return init_.plugins_root


def get_plugin_names() -> typing.List[str]:
    if init_.plugins_root is None:
        return []
    plugin_names = []
    for s in init_.plugins_info_map:
        plugin_names.append(s)

    plugin_names.sort()
    return plugin_names


def get_info(plugin_name: str) -> typing.Optional[dict]:
    if init_.plugins_root is None:
        return None

    plugin_name = _normalize_plugin_name(plugin_name)

    if plugin_name not in init_.plugins_info_map:
        return None
    return init_.plugins_info_map[plugin_name]


def get_source_code(plugin_name: str) -> typing.Optional[str]:
    if init_.plugins_root is None:
        return None

    plugin_name = _normalize_plugin_name(plugin_name)

    if plugin_name not in init_.plugins_info_map:
        return None

    code = ""
    with open(f"{init_.plugins_root}/{plugin_name}/plugin.py") as f:
        code += f.read()
    return code


def get_plugin(plugin_name: str) -> typing.Optional[any]:
    if init_.plugins_root is None:
        return None

    plugin_name = _normalize_plugin_name(plugin_name)
    return _get_plugin(init_.plugins_root, plugin_name)


# -----------------------------------------------------------------------------
# UTILS
# -----------------------------------------------------------------------------


def init_plugin(plugin_name: str):
    if init_.plugins_root is None:
        return None

    plugin_name = _normalize_plugin_name(plugin_name)

    if not _is_valid_plugin_name(plugin_name):
        print(f'WARN: init_plugin("{plugin_name}") failed, invalid name')
        return

    if os.path.exists(f"{init_.plugins_root}/{plugin_name}/plugin.py"):
        print(f'WARN: init_plugin("{plugin_name}") failed, plugin exists')
        return

    golden_plugin_skectch_code = plugin_template.get_plugin_template_code(plugin_name)
    golden_plugin_skectch_code_test = plugin_template.get_plugin_test_template_code(
        plugin_name
    )

    if not os.path.exists(f"{init_.plugins_root}/{plugin_name}"):
        os.makedirs(f"{init_.plugins_root}/{plugin_name}")

    with open(f"{init_.plugins_root}/{plugin_name}/plugin.py", "w") as file:
        file.write(golden_plugin_skectch_code)

    with open(f"{init_.plugins_root}/{plugin_name}/plugin_test.py", "w") as file:
        file.write(golden_plugin_skectch_code_test)

    gendoc(plugin_name)


def gendoc(plugin_name: str):
    if init_.plugins_root is None:
        print("WARN: plugin root not found")
        return

    if not _is_valid_plugin_name(plugin_name):
        print(f'WARN: gendoc("{plugin_name}") failed, invalid name')
        return

    pkg = _get_plugin(init_.plugins_root, plugin_name)
    if not pkg:
        print("WARN: plugin init failed")
        return
    info = _get_info(pkg)
    if info is None:
        print(f'WARN: gendoc("{plugin_name}") failed, not found plugin')
        return

    with open(f"{init_.plugins_root}/{plugin_name}/api.md", "w") as file:
        file.write(f"# plugin: `{info['name']}` - {info['describe']}\n\n")
        file.write(f"{info['long_describe']}\n\n")
        file.write(f"*version: {info['version']}*\n\n")

        for func_name in info["method"]:
            func_doc = info["method"][func_name]

            func_doc_line = []
            for line in func_doc.splitlines():
                if line.startswith("    "):
                    line = line[4:]
                func_doc_line.append(line)

            file.write(f"## `{func_name}`\n\n")
            for line in func_doc_line:
                file.write(f"{line}\n")

            file.write("\n")

    return


# -----------------------------------------------------------------------------
# END
# -----------------------------------------------------------------------------
