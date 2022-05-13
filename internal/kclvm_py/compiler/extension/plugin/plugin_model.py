from typing import Callable, Optional, Dict
from functools import wraps
import inspect

import kclvm.kcl.info as kcl_info

from .plugin import get_plugin_names, get_plugin
from kclvm.api.object import KCLBuiltinFunctionObject


PLUGIN_MODULE_NAME = "kcl_plugin."
STANDARD_SYSTEM_MODULE_LOCATION = "kclvm_plugin"


def kcl_plugin(func):
    @wraps(func)
    def decorated(*args, **kwargs):
        return func(*args, **kwargs)

    return decorated


def new_plugin_function(
    name: str, func: Callable
) -> Optional[KCLBuiltinFunctionObject]:
    """New a plugin function object using a native plugin function"""
    if not func or not name:
        return None
    return KCLBuiltinFunctionObject(name=name, function=func)


def get_plugin_func_objects(plugin_name: str) -> Dict[str, KCLBuiltinFunctionObject]:
    """Get all plugin function objects from a plugin named 'plugin_name'"""
    if (
        not plugin_name
        or plugin_name.replace(PLUGIN_MODULE_NAME, "") not in get_plugin_names()
    ):
        return {}
    module = get_plugin(plugin_name)
    members = inspect.getmembers(module)
    result = {
        kcl_info.demangle(func_name): new_plugin_function(
            kcl_info.demangle(func_name), func
        )
        for func_name, func in members
        if kcl_info.ismangled(func_name)
    }
    return result
