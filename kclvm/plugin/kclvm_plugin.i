// Copyright 2021 The KCL Authors. All rights reserved.

// http://www.swig.org/Doc3.0/Python.html#Python_directors

% module(directors = "1") kclvm_plugin

    % {
#define SWIG_FILE_WITH_INIT
#include "kclvm_plugin.h"
          % }

    // ----------------------------------------------------------------------------
    // C/C++ code
    // ----------------------------------------------------------------------------

    % include "stdint.i" % include "std_string.i"

    % feature("director") _kclvm_plugin_AppContextBase;
class _kclvm_plugin_AppContextBase
{
public:
    _kclvm_plugin_AppContextBase(uint64_t rust_invoke_json_ptr);
    virtual ~_kclvm_plugin_AppContextBase();

    void _clear_options();
    void _add_option(const std::string &key, const std::string &value);

    std::string _run_app(
        uint64_t _start_fn_ptr,
        uint64_t _kclvm_main_ptr, // main.k => kclvm_main
        int32_t strict_range_check,
        int32_t disable_none,
        int32_t disable_schema_check,
        int32_t list_option_mode,
        int32_t debug_mode,
        int32_t buffer_size);

    std::string _get_warn();

    uint64_t _get_cxx_invoke_proxy_ptr();

    std::string _call_rust_method(
        const std::string &name,
        const std::string &args_json,
        const std::string &kwargs_json);

    virtual std::string _call_py_method(
        const std::string &name,
        const std::string &args_json,
        const std::string &kwargs_json);
};

// ----------------------------------------------------------------------------
// Python code
// ----------------------------------------------------------------------------

% pythonbegin % {import sys import typing import ctypes import os import importlib import json import inspect

                     import kclvm.kcl.info as kcl_info import kclvm.compiler.extension.plugin.plugin as kcl_plugin import kclvm.api.object.internal.option as option import kclvm.api.object as objpkg % }

    % pythoncode %
{
class AppContext(_kclvm_plugin_AppContextBase):
    def __init__(self, app_dll_name: str):
        self._is_windows: bool = os.name == "nt"

        self._start_func_name: str = ""
        self._app_dll_name = app_dll_name
        self._plugin_dict: typing.Dict[str, any] = {}

        if self._is_windows:
            _executable_root = os.path.dirname(sys.executable)
            self._kclvm_runtime = ctypes.CDLL(f"{_executable_root}\\kclvm_cli_cdylib.dll")
            self._app_lib = ctypes.CDLL(app_dll_name)
        else:
            self._kclvm_runtime = ctypes.CDLL(app_dll_name)
            self._app_lib = ctypes.CDLL(app_dll_name)

        self._kclvm_runtime.kclvm_plugin_init.restype = None
        self._kclvm_runtime.kclvm_plugin_init.argtypes = [ctypes.c_longlong]

        self._kclvm_runtime.kclvm_plugin_invoke_json.restype = ctypes.c_char_p
        self._kclvm_runtime.kclvm_plugin_invoke_json.argtypes = [
            ctypes.c_char_p,
            ctypes.c_char_p,
            ctypes.c_char_p
        ]

        rust_invoke_json_ptr = ctypes.cast(self._kclvm_runtime.kclvm_plugin_invoke_json, ctypes.c_void_p).value
        super().__init__(rust_invoke_json_ptr)

        self._kclvm_runtime.kclvm_plugin_init(self._get_cxx_invoke_proxy_ptr())

    def InitOptions(self, arguments):
        self._clear_options()
        for kv in arguments or []:
            key, value = kv
            if isinstance(value, (bool, list, dict)):
                value = json.dumps(value)
            elif isinstance(value, str):
                value = '"{}"'.format(value.replace('"', '\\"'))
            else:
                value = str(value)
            self._add_option(key, value)

    def RunApp(self, *,
        start_func_name='_kcl_run',
        strict_range_check=None,
        disable_none=None,
        disable_schema_check=None,
        list_option_mode=None,
        debug_mode=None,
        buffer_size=0
    ) -> str:
        self._start_func_name = start_func_name

        _start = getattr(self._kclvm_runtime, start_func_name)
        _start_ptr = ctypes.cast(_start, ctypes.c_void_p).value

        if hasattr(self._app_lib, 'kclvm_main'):
            _kclvm_main = getattr(self._app_lib, 'kclvm_main')
            _kclvm_main_ptr = ctypes.cast(_kclvm_main, ctypes.c_void_p).value
        elif hasattr(self._app_lib, 'kclvm_main_win'):
            _kclvm_main = getattr(self._app_lib, 'kclvm_main_win')
            _kclvm_main_ptr = ctypes.cast(_kclvm_main, ctypes.c_void_p).value
        else:
            _kclvm_main_ptr = 0

        if disable_none:
            disable_none = 1
        else:
            disable_none = 0

        if strict_range_check:
            strict_range_check = 1
        else:
            strict_range_check = 0

        if disable_schema_check:
            disable_schema_check = 1
        else:
            disable_schema_check = 0

        if list_option_mode:
            list_option_mode = 1
        else:
            list_option_mode = 0

        if debug_mode:
            debug_mode = 1
        else:
            debug_mode = 0

        json_result = self._run_app(_start_ptr, _kclvm_main_ptr,
            strict_range_check,
            disable_none,
            disable_schema_check,
            list_option_mode,
            debug_mode,
            buffer_size
        )
        return json_result

    def GetWarn(self) -> str:
        json_warn_result = self._get_warn()
        return json_warn_result

    def CallMethod(self, name:str, args_json:str, kwargs_json:str) -> str:
        return self._call_rust_method(name, args_json, kwargs_json)

    def _call_py_method(self, name:str, args_json:str, kwargs_json:str) -> str:
try:
            return self._call_py_method_unsafe(name, args_json, kwargs_json)
        except Exception as e:
            return json.dumps({ "__kcl_PanicInfo__": f"{e}" })

    def _get_plugin(self, plugin_name:str) -> typing.Optional[any]:
        if plugin_name in self._plugin_dict:
            return self._plugin_dict[plugin_name]

        module = kcl_plugin.get_plugin(plugin_name)
        self._plugin_dict[plugin_name] = module
        return module

    def _call_py_method_unsafe(self, name:str, args_json:str, kwargs_json:str) -> str:
        dotIdx = name.rfind(".")
        if dotIdx < 0:
            return ""

        modulePath = name[:dotIdx]
        mathodName = name[dotIdx+1:]

        plugin_name = modulePath[modulePath.rfind('.')+1:]

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

    def __del__(self):
        self._free_library()

    def _free_library(self):
        if os.name == 'nt':
            import ctypes.wintypes
            kernel32 = ctypes.WinDLL('kernel32', use_last_error=True)
            kernel32.FreeLibrary.argtypes = [ctypes.wintypes.HMODULE]
            kernel32.FreeLibrary(self._app_lib._handle)
            self._app_lib = None
#kernel32 = ctypes.WinDLL('kernel32', use_last_error = True)
#kernel32.FreeLibrary.argtypes = [ctypes.wintypes.HMODULE]
#kernel32.FreeLibrary(self._app_dll._handle)
            pass
        else:
#libdl = ctypes.CDLL("libdl.so")
#libdl.dlclose(self._app_dll._handle)
           pass


def main(args: typing.List[str]):
    if len(args) < 2:
        print("usage: kclvm_plugin app.[dll|dylib|so]")
        sys._exit(1)

    ctx = AppContext(args[1])
    ctx.RunApp(args[2:])


if __name__ == "__main__":
    main(sys.argv)
%
}

// ----------------------------------------------------------------------------
// END
// ----------------------------------------------------------------------------
