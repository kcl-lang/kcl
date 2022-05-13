# Copyright 2020 The KCL Authors. All rights reserved.

import unittest

import kclvm.api.object.internal.common as common


class TestCommonInternal(unittest.TestCase):
    def test_is_builtin_type(self):
        cases = [
            # True cases
            {"type": "int", "expected": True},
            {"type": "float", "expected": True},
            {"type": "str", "expected": True},
            {"type": "bool", "expected": True},
            # False cases
            {"type": "", "expected": False},
            {"type": "int8", "expected": False},
            {"type": "string", "expected": False},
            {"type": "{:}", "expected": False},
            {"type": "[]", "expected": False},
            {"type": "pkg.Schema", "expected": False},
        ]
        for case in cases:
            tpe_str, expected = case["type"], case["expected"]
            value = common.is_builtin_type(tpe_str)
            self.assertEqual(value, expected)

    def test_is_dict_type(self):
        cases = [
            # True cases
            {"type": "{}", "expected": True},
            {"type": "{:}", "expected": True},
            {"type": "{str:}", "expected": True},
            {"type": "{str:str}", "expected": True},
            {"type": "{str: str}", "expected": True},
            {"type": "{str:int}", "expected": True},
            {"type": "{str: int}", "expected": True},
            {"type": "{str:{str:}}", "expected": True},
            {"type": "{str:{str:str}}", "expected": True},
            {"type": "{str:str|int}", "expected": True},
            {"type": "{str:pkg.Schema}", "expected": True},
            # False cases
            {"type": "int", "expected": False},
            {"type": "float", "expected": False},
            {"type": "str", "expected": False},
            {"type": "bool", "expected": False},
            {"type": "", "expected": False},
            {"type": "int8", "expected": False},
            {"type": "string", "expected": False},
            {"type": "{", "expected": False},
            {"type": "}", "expected": False},
        ]
        for case in cases:
            tpe_str, expected = case["type"], case["expected"]
            value = common.isdicttype(tpe_str)
            self.assertEqual(value, expected)

    def test_is_list_type(self):
        cases = [
            # True cases
            {"type": "[]", "expected": True},
            {"type": "[int]", "expected": True},
            {"type": "[str]", "expected": True},
            {"type": "[[str]]", "expected": True},
            {"type": "[str|int]", "expected": True},
            {"type": "[pkg.Schema]", "expected": True},
            # False cases
            {"type": "int", "expected": False},
            {"type": "float", "expected": False},
            {"type": "str", "expected": False},
            {"type": "bool", "expected": False},
            {"type": "", "expected": False},
            {"type": "int8", "expected": False},
            {"type": "string", "expected": False},
            {"type": "[", "expected": False},
            {"type": "]", "expected": False},
        ]
        for case in cases:
            tpe_str, expected = case["type"], case["expected"]
            value = common.islisttype(tpe_str)
            self.assertEqual(value, expected)

    def test_separate_kv(self):
        cases = [
            {"type": "{}", "expected": ("", "")},
            {"type": "{:}", "expected": ("", "")},
            {"type": "{str:}", "expected": ("str", "")},
            {"type": "{str:str}", "expected": ("str", "str")},
            {"type": "{str:int}", "expected": ("str", "int")},
            {"type": "{str:{str:}}", "expected": ("str", "{str:}")},
            {"type": "{str:{str:str}}", "expected": ("str", "{str:str}")},
            {"type": "{str:str|int}", "expected": ("str", "str|int")},
            {"type": "{str:pkg.Schema}", "expected": ("str", "pkg.Schema")},
        ]
        for case in cases:
            tpe_str, expected = case["type"], case["expected"]
            value = common.separate_kv(common.dereferencetype(tpe_str))
            self.assertEqual(value, expected, msg=case["type"])

    def test_union_native(self):
        cases = [
            # Left None
            {"value1": None, "value2": 1, "expected": 1},
            {"value1": None, "value2": 1.1, "expected": 1.1},
            {"value1": None, "value2": [], "expected": []},
            {"value1": None, "value2": {}, "expected": {}},
            {"value1": None, "value2": "s", "expected": "s"},
            {"value1": None, "value2": True, "expected": True},
            {"value1": None, "value2": False, "expected": False},
            {"value1": None, "value2": None, "expected": None},
            # Right None
            {"value1": 1, "value2": None, "expected": 1},
            {"value1": 1.1, "value2": None, "expected": 1.1},
            {"value1": [], "value2": None, "expected": []},
            {"value1": {}, "value2": None, "expected": {}},
            {"value1": "s", "value2": None, "expected": "s"},
            {"value1": True, "value2": None, "expected": True},
            {"value1": False, "value2": None, "expected": False},
            {"value1": None, "value2": None, "expected": None},
            # Int
            {"value1": 1, "value2": 1, "expected": 1},
            {"value1": 1, "value2": 2, "expected": 2},
            {"value1": 1, "value2": 3, "expected": 3},
            # Float
            {"value1": 1.0, "value2": 1.0, "expected": 1.0},
            {"value1": 1.0, "value2": 1.5, "expected": 1.5},
            # String
            {"value1": "s", "value2": "", "expected": ""},
            {"value1": "s", "value2": "s", "expected": "s"},
            {"value1": "s", "value2": "ss", "expected": "ss"},
            # Boolean True
            {"value1": True, "value2": True, "expected": True},
            {"value1": True, "value2": False, "expected": False},
            # Boolean False
            {"value1": False, "value2": False, "expected": False},
            {"value1": False, "value2": True, "expected": True},
            # List
            {"value1": [], "value2": [], "expected": []},
            {"value1": [], "value2": [1], "expected": [1]},
            {"value1": [], "value2": [1, 2], "expected": [1, 2]},
            {"value1": [1], "value2": [1], "expected": [1]},
            {"value1": [1], "value2": [2], "expected": [2]},
            {"value1": [1], "value2": [2, 2], "expected": [2, 2]},
            {"value1": [1, 2], "value2": [3, 4], "expected": [3, 4]},
            {"value1": [1, 2, 3], "value2": [3, 4], "expected": [3, 4, 3]},
            {
                "value1": [{"key1": "value1"}],
                "value2": [{"key1": "value1"}],
                "expected": [{"key1": "value1"}],
            },
            {
                "value1": [{"key1": "value1"}],
                "value2": [{"key1": "value2"}],
                "expected": [{"key1": "value2"}],
            },
            {
                "value1": [{"key1": "value1"}],
                "value2": [{"key2": "value2"}],
                "expected": [{"key1": "value1", "key2": "value2"}],
            },
            {
                "value1": [{"key1": "value1"}],
                "value2": [{"key1": "value1", "key2": "value2"}],
                "expected": [{"key1": "value1", "key2": "value2"}],
            },
            {
                "value1": [{"key1": "value1"}],
                "value2": [{"key2": "value2", "key1": "value1"}],
                "expected": [{"key1": "value1", "key2": "value2"}],
            },
            {
                "value1": [{"key1": "value1", "key2": "value2"}],
                "value2": [{"key1": "value1"}],
                "expected": [{"key1": "value1", "key2": "value2"}],
            },
            # Dict
            {"value1": {}, "value2": {}, "expected": {}},
            {
                "value1": {},
                "value2": {"key1": "value1"},
                "expected": {"key1": "value1"},
            },
            {
                "value1": {"key1": "value1"},
                "value2": {},
                "expected": {"key1": "value1"},
            },
            {
                "value1": {"key1": "value1"},
                "value2": {"key1": "value1"},
                "expected": {"key1": "value1"},
            },
            {
                "value1": {"key1": "value1"},
                "value2": {"key1": "value1", "key2": "value2"},
                "expected": {"key1": "value1", "key2": "value2"},
            },
            {
                "value1": {"key1": "value1", "key2": "value2"},
                "value2": {"key1": "value1"},
                "expected": {"key1": "value1", "key2": "value2"},
            },
            {
                "value1": {"s": {"key1": "value1"}},
                "value2": {"s": {"key1": "value1", "key2": "value2"}},
                "expected": {"s": {"key1": "value1", "key2": "value2"}},
            },
            {
                "value1": {"s": {"key1": "value1", "key2": "value2"}},
                "value2": {"s": {"key1": "value1"}},
                "expected": {"s": {"key1": "value1", "key2": "value2"}},
            },
            {
                "value1": {"key2": "value2", "key1": "value1"},
                "value2": {"key1": "value1", "key2": "value2"},
                "expected": {"key1": "value1", "key2": "value2"},
            },
            {
                "value1": {"key2": "value2", "key1": "value1"},
                "value2": {"key1": "value1", "key2": "value2"},
                "expected": {"key1": "value1", "key2": "value2"},
            },
        ]
        for case in cases:
            value1, value2, expected = case["value1"], case["value2"], case["expected"]
            union_value = common.union(value1, value2)
            self.assertEqual(union_value, expected)


if __name__ == "__main__":
    unittest.main(verbosity=2)
