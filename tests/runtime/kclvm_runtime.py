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

        # kclvm_context_t* kclvm_context_new();
        self._app_lib.kclvm_context_new.restype = ctypes.c_void_p

        # void kclvm_context_delete(kclvm_context_t* p);
        self._app_lib.kclvm_context_delete.argtypes = [
            ctypes.c_void_p,
        ]

        # const char* kclvm_context_invoke(kclvm_context_t* p, const char* method, const char* args, const char* kwargs);
        self._app_lib.kclvm_context_invoke.restype = ctypes.c_char_p
        self._app_lib.kclvm_context_invoke.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_char_p,
            ctypes.c_char_p,
        ]

    def _kclvm_context_new(self) -> ctypes.c_void_p:
        return self._app_lib.kclvm_context_new()

    def kclvm_context_delete(self, ctx: ctypes.c_void_p):
        self._app_lib.kclvm_context_delete(ctx)

    def _kclvm_context_invoke(
        self, ctx: ctypes.c_void_p, method: str, args: str, kwargs: str
    ) -> any:
        jsonValue = self._app_lib.kclvm_context_invoke(
            ctx, method.encode(), args.encode(), kwargs.encode()
        )
        return json.loads(jsonValue)

    def Path(self) -> str:
        return self.dllpath

    def Invoke(self, method: str, *args, **kwargs) -> any:
        if self.ctx is None:
            self.ctx = self._kclvm_context_new()

        if not method.startswith("kclvm_"):
            if method.startswith("str."):
                # str.startswith => kclvm_builtin_str_startswith
                method = f'kclvm_builtin_{method.replace(".", "_")}'
            elif "." in method:
                # regex.match => kclvm_regex_match
                method = f'kclvm_{method.replace(".", "_")}'  # json.encode => kclvm_json_encode
            else:
                method = f"kclvm_builtin_{method}"  # print => kclvm_builtin_print

        return self._kclvm_context_invoke(
            self.ctx, method, json.dumps(args), json.dumps(kwargs)
        )


if __name__ == "__main__":
    dylib = KclRuntimeDylib()
    dylib.Invoke(f"print", "hello kclvm")

