# Copyright 2020 The KCL Authors. All rights reserved.

import unittest

import kclvm.config
import kclvm.kcl.info as kcl_info
import kclvm.kcl.error as kcl_error
import kclvm.api.object as objpkg

from kclvm.compiler.check.check_type import check


class TestRangeCheck(unittest.TestCase):
    def test_object_range_check_normal(self):
        kclvm.config.debug = True
        cases = [
            1,
            2.0,
            kcl_info.INT32_MAX + 1,
            kcl_info.INT64_MAX,
            kcl_info.FLOAT32_MAX,
        ]
        for case in cases:
            check(objpkg.to_kcl_obj(case))
        kclvm.config.debug = False

    def test_object_range_check_invalid(self):
        kclvm.config.debug = True
        kclvm.config.strict_range_check = True
        cases = [
            kcl_info.INT32_MAX + 1,
            kcl_info.FLOAT32_MAX * 2,
        ]
        for case in cases:
            with self.assertRaises(kcl_error.KCLException):
                check(objpkg.to_kcl_obj(case))
        kclvm.config.strict_range_check = False
        cases = [
            kcl_info.INT64_MAX + 1,
            kcl_info.FLOAT64_MAX * 2,
        ]
        for case in cases:
            with self.assertRaises(kcl_error.KCLException):
                check(objpkg.to_kcl_obj(case))
        kclvm.config.debug = False


if __name__ == "__main__":
    unittest.main(verbosity=2)
