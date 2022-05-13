# Copyright 2020 The KCL Authors. All rights reserved.

import unittest

import kclvm.compiler.extension.builtin.system_module.units as units


class TestUnitsSystemModule(unittest.TestCase):
    def test_to_unit(self):
        cases = [
            {"num": 1e9, "suffix": "G", "expected": "1G"},
            {"num": 1e10, "suffix": "G", "expected": "10G"},
            {"num": 1e9, "suffix": "M", "expected": "1000M"},
        ]
        for case in cases:
            num, suffix, expected = case["num"], case["suffix"], case["expected"]
            self.assertEqual(units.to_unit(num, suffix), expected)


if __name__ == "__main__":
    unittest.main(verbosity=2)
