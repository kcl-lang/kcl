# Copyright 2021 The KCL Authors. All rights reserved.

import typing
import unittest

import kclvm_runtime

# https://github.com/python/cpython/blob/main/Lib/test

_Dylib = kclvm_runtime.KclvmRuntimeDylib()


class kclx_Crypto:
    def __init__(self, dylib_=None):
        self.dylib = dylib_ if dylib_ else _Dylib


class BaseTest(unittest.TestCase):
    def test_foo(self):
        self.assertTrue(True)


if __name__ == "__main__":
    unittest.main()
