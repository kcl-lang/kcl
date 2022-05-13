# Copyright 2020 The KCL Authors. All rights reserved.

import pathlib
import unittest

import kclvm.kcl.error as kcl_error
import kclvm.config as kcfg
import kclvm.api.object as objpkg
import kclvm.api.object.internal.path_selector as path_selector


class TestBytecodeObject(unittest.TestCase):
    def setUp(self):
        kcfg.path_selector = [["", "data0"]]
        return super().setUp()

    def tearDown(self):
        kcfg.path_selector = []
        return super().tearDown()

    def test_kcl_result(self):
        filename = str(
            pathlib.Path(__file__)
            .parent.joinpath("path_selector_test_data")
            .joinpath("main.k")
        )
        data = {"data0": {"id": 0}, "data1": {"id": 1}, "data2": {"id": 2}}
        dict_obj = objpkg.to_kcl_obj(data).value
        result = objpkg.KCLResult(dict_obj, filename)
        self.assertEqual(str(result), str(dict_obj))
        self.assertEqual(
            result.filter_by_path_selector(),
            objpkg.to_kcl_obj({"data0": {"id": 0}}).value,
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
