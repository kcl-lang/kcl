# Copyright The KCL Authors. All rights reserved.

import sys
import os
import unittest

# Add the parent directory to the path to import kcl_runtime
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.append(parent_dir)

import kcl_runtime as kcl_runtime

# https://github.com/python/cpython/blob/main/Lib/test

_Dylib = kcl_runtime.KclRuntimeDylib()


class kclx_Datetime:
    def __init__(self, dylib_=None):
        self.dylib = dylib_ if dylib_ else _Dylib

    def is_rfc3339(self, date: str) -> bool:
        return self.dylib.Invoke("datetime.is_rfc3339", date)

    def is_iso8601(self, date: str) -> bool:
        return self.dylib.Invoke("datetime.is_iso8601", date)


kclxdatetime = kclx_Datetime(_Dylib)


class BaseTest(unittest.TestCase):
    def test_is_rfc3339(self):
        self.assertTrue(kclxdatetime.is_rfc3339("2024-03-20T15:30:00Z"))
        self.assertTrue(kclxdatetime.is_rfc3339("2024-03-20T15:30:00+08:00"))
        self.assertFalse(kclxdatetime.is_rfc3339("2024-03-20"))

    def test_is_iso8601(self):
        self.assertTrue(kclxdatetime.is_iso8601("2024-03-20T15:30:00Z"))
        self.assertTrue(kclxdatetime.is_iso8601("P3Y6M4DT12H30M5S"))
        self.assertTrue(kclxdatetime.is_iso8601("P1Y"))
        self.assertFalse(kclxdatetime.is_iso8601("invalid"))


if __name__ == "__main__":
    unittest.main()

