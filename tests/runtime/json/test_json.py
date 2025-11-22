# Copyright The KCL Authors. All rights reserved.

import typing
import unittest

import tests.runtime.kcl_runtime as kcl_runtime

# https://github.com/python/cpython/blob/main/Lib/test

_Dylib = kcl_runtime.KclRuntimeDylib()


class kclx_Json:
    def __init__(self, dylib_=None):
        self.dylib = dylib_ if dylib_ else _Dylib


class BaseTest(unittest.TestCase):
    def test_foo(self):
        self.assertTrue(True)


if __name__ == "__main__":
    unittest.main()
