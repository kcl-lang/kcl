# Copyright 2020 The KCL Authors. All rights reserved.

import unittest

import kclvm.api.object.internal.path_selector as ps


class TestPathSelector(unittest.TestCase):
    def test_select_instance_attributes(self):
        cases = [
            {"inst": {"key1": "value1"}, "attrs": None, "expected": {"key1": "value1"}},
            {"inst": {"key1": "value1"}, "attrs": {}, "expected": {"key1": "value1"}},
            {
                "inst": {"key1": "value1"},
                "attrs": {"key1": {}},
                "expected": {"key1": "value1"},
            },
            {
                "inst": {"key1": "value1"},
                "attrs": {"err_key": {}},
                "expected": None,
            },
            {
                "inst": {"key1": {"internal_key": "value1"}},
                "attrs": {"key1": {}},
                "expected": {"key1": {"internal_key": "value1"}},
            },
            {
                "inst": {"key1": {"internal_key": "value1"}},
                "attrs": {"key1": {"internal_key": {}}},
                "expected": {"key1": {"internal_key": "value1"}},
            },
            {
                "inst": {"key1": "value1", "key2": "value2"},
                "attrs": {"key1": {}},
                "expected": {"key1": "value1"},
            },
        ]
        for case in cases:
            inst, attrs, expected = case["inst"], case["attrs"], case["expected"]
            value = ps.select_instance_attributes(inst, attrs)
            self.assertEqual(value, expected)


if __name__ == "__main__":
    unittest.main(verbosity=2)
