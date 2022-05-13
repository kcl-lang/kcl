# Copyright 2020 The KCL Authors. All rights reserved.

import unittest
import pathlib

import kclvm.kcl.error as kcl_error
import kclvm.compiler.vfs as vfs

CURRENT_PATH = str(pathlib.Path(__file__).parent)


class TestVfsGetRoot(unittest.TestCase):
    def test_get_pkg_root(self):
        cases = [
            {"path": "../..", "expected": None},
            {"path": f"{CURRENT_PATH}/pkg", "expected": str(CURRENT_PATH)},
            {"path": f"{CURRENT_PATH}/test_get_pkg_root", "expected": f"{CURRENT_PATH}/test_get_pkg_root"},
        ]
        for case in cases:
            path, expected = case["path"], case["expected"]
            result = vfs.GetPkgRoot(path)
            self.assertEqual(result, expected)

    def test_must_get_pkg_root(self):
        cases = [
            {"paths": ["../..", "."], "expected": None},
            {"paths": [f"{CURRENT_PATH}/pkg", "."], "expected": str(CURRENT_PATH)},
        ]
        for case in cases:
            paths, expected = case["paths"], case["expected"]
            result = vfs.MustGetPkgRoot(paths)
            self.assertEqual(result, expected)

    def test_must_get_pkg_root_invalid(self):
        cases = [
            {"paths": [f"{CURRENT_PATH}/test_get_pkg_root", str(CURRENT_PATH)]},
        ]
        for case in cases:
            paths = case["paths"]
            with self.assertRaises(kcl_error.CompileError):
                result = vfs.MustGetPkgRoot(paths)


if __name__ == "__main__":
    unittest.main(verbosity=2)
