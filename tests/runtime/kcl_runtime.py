# Copyright The KCL Authors. All rights reserved.

import ctypes
import json
import os
import sys


def _find_default_dylib_path() -> str:
    _executable_root = os.path.dirname(os.environ.get("KCL_LIB_PATH") or sys.executable)

    pathList = [
        f"{_executable_root}/lib/libkcl.dylib",
        f"{_executable_root}/lib/libkcl.so",
        f"{_executable_root}/bin/libkcl.dylib",
        f"{_executable_root}/bin/libkcl.so",
        f"{_executable_root}/libkcl.dylib",
        f"{_executable_root}/libkcl.so",
        f"{_executable_root}\\bin\\kcl.dll",
        f"{_executable_root}\\kcl.dll",
        f"{os.path.dirname(__file__)}/../../target/release/libkcl.dylib",
        f"{os.path.dirname(__file__)}/../../target/release/libkcl.so",
        f"{os.path.dirname(__file__)}\\..\\..\\target\\release\\kcl.dll",
        f"{os.path.dirname(__file__)}/../../target/debug/libkcl.dylib",
        f"{os.path.dirname(__file__)}/../../target/debug/libkcl.so",
        f"{os.path.dirname(__file__)}\\..\\..\\target\\debug\\kcl.dll",
    ]

    for s in pathList:
        if os.path.exists(s):
            return s
    return ""


class KclRuntimeDylib:
    def __init__(self, dllpath: str = None):
        if dllpath is None:
            dllpath = _find_default_dylib_path()
        if not dllpath:
            raise f"kcl runtime lib not found"

        self.dllpath = dllpath
        self._app_dll = ctypes.cdll.LoadLibrary(dllpath)
        self._app_lib = ctypes.CDLL(dllpath)
        self.ctx = None

        # kcl_context_t* kcl_context_new();
        self._app_lib.kcl_context_new.restype = ctypes.c_void_p

        # void kcl_context_delete(kcl_context_t* p);
        self._app_lib.kcl_context_delete.argtypes = [
            ctypes.c_void_p,
        ]

        # const char* kcl_context_invoke(kcl_context_t* p, const char* method, const char* args, const char* kwargs);
        self._app_lib.kcl_context_invoke.restype = ctypes.c_char_p
        self._app_lib.kcl_context_invoke.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_char_p,
            ctypes.c_char_p,
        ]

    def _kcl_context_new(self) -> ctypes.c_void_p:
        return self._app_lib.kcl_context_new()

    def kcl_context_delete(self, ctx: ctypes.c_void_p):
        self._app_lib.kcl_context_delete(ctx)

    def _kcl_context_invoke(
        self, ctx: ctypes.c_void_p, method: str, args: str, kwargs: str
    ) -> any:
        jsonValue = self._app_lib.kcl_context_invoke(
            ctx, method.encode(), args.encode(), kwargs.encode()
        )
        return json.loads(jsonValue)

    def Path(self) -> str:
        return self.dllpath

    def Invoke(self, method: str, *args, **kwargs) -> any:
        if self.ctx is None:
            self.ctx = self._kcl_context_new()

        if not method.startswith("kcl_"):
            if method.startswith("str."):
                # str.startswith => kcl_builtin_str_startswith
                method = f'kcl_builtin_{method.replace(".", "_")}'
            elif "." in method:
                # regex.match => kcl_regex_match
                method = (
                    f'kcl_{method.replace(".", "_")}'  # json.encode => kcl_json_encode
                )
            else:
                method = f"kcl_builtin_{method}"  # print => kcl_builtin_print

        return self._kcl_context_invoke(
            self.ctx, method, json.dumps(args), json.dumps(kwargs)
        )


if __name__ == "__main__":
    dylib = KclRuntimeDylib()
    dylib.Invoke(f"print", "hello kcl")
