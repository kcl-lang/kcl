# Copyright 2020 The KCL Authors. All rights reserved.

import unittest
from typing import cast

import kclvm.kcl.ast as ast
import kclvm.kcl.error as kcl_error
from kclvm.api.object import (
    KCLIntObject,
    KCLNumberMultiplierObject,
    KCLDictObject,
    KCLSchemaConfigObject,
    KCLSchemaObject,
    Undefined,
    to_kcl_obj,
    to_python_obj,
)


def to_kcl_dict_obj(data: dict) -> KCLDictObject:
    return cast(KCLDictObject, to_kcl_obj(data))


class TestNumberMultiplierObject(unittest.TestCase):
    def test_number_multiplier_member_function(self):
        obj = KCLNumberMultiplierObject(
            value=1024,
            raw_value=1,
            binary_suffix="Mi",
        )
        self.assertEqual(obj.type_str(), "number_multiplier(1Mi)")
        self.assertEqual(str(obj), "1Mi")
        self.assertEqual(repr(obj), "1Mi")
        self.assertEqual(int(obj), 1024)
        self.assertEqual(float(obj), 1024.0)
        self.assertEqual(bool(obj), True)


class TestDictObject(unittest.TestCase):
    def test_dict_object_append_unpack(self):
        cases = [
            {
                "data": {"key1": 1},
                "item": {"key2": 2},
                "expected": {"key1": 1, "key2": 2},
            },
            {"data": {"key1": 1}, "item": None, "expected": {"key1": 1}},
            {"data": {"key1": 1}, "item": Undefined, "expected": {"key1": 1}},
            {
                "data": {"key1": 1},
                "item": KCLSchemaConfigObject(
                    value={"key1": KCLIntObject(2)},
                    operation_map={"key1": ast.ConfigEntryOperation.OVERRIDE},
                ),
                "expected": {"key1": 2},
            },
            {
                "data": {"key1": 1},
                "item": KCLSchemaObject(
                    attrs={"key1": KCLIntObject(2)},
                    operation_map={"key1": ast.ConfigEntryOperation.OVERRIDE},
                ),
                "expected": {"key1": 2},
            },
        ]
        for case in cases:
            data, item, expected = (
                to_kcl_obj(case["data"]),
                to_kcl_obj(case["item"]),
                to_kcl_obj(case["expected"]),
            )
            data.append_unpack(item)
            self.assertEqual(to_python_obj(data), to_python_obj(expected))

    def test_dict_object_insert_with_key(self):
        cases = [
            {
                "data": {"key": []},
                "key": "key",
                "value": [1],
                "index": None,
                "expected": {"key": [1]},
            },
            {
                "data": {"key": []},
                "key": "key",
                "value": [1],
                "index": -1,
                "expected": {"key": [1]},
            },
            {
                "data": {"key": [0]},
                "key": "key",
                "value": [1],
                "index": -1,
                "expected": {"key": [0, 1]},
            },
            {
                "data": {"key": [0]},
                "key": "key",
                "value": [1],
                "index": 0,
                "expected": {"key": [1, 0]},
            },
            {
                "data": {"key": None},
                "key": "key",
                "value": [1],
                "index": -1,
                "expected": {"key": [1]},
            },
        ]
        invalid_cases = [
            {
                "data": {"key": 1},
                "key": "key",
                "value": [1],
                "index": -1,
                "expected": {"key": [1]},
            },
        ]
        for case in cases:
            data, key, value, index, expected = (
                to_kcl_dict_obj(case["data"]),
                case["key"],
                to_kcl_obj(case["value"]),
                case["index"],
                case["expected"],
            )
            data.insert_with_key(key, value, index)
            self.assertEqual(to_python_obj(data), expected)
        for case in invalid_cases:
            data, key, value, index, expected = (
                to_kcl_dict_obj(case["data"]),
                case["key"],
                to_kcl_obj(case["value"]),
                case["index"],
                case["expected"],
            )
            with self.assertRaises(kcl_error.KCLException):
                data.insert_with_key(key, value, index)

    def test_dict_object_list_key_override(self):
        cases = [
            {
                "data": {"key": [0]},
                "attr": "key",
                "value": 1,
                "index": 0,
                "expected": {"key": [1]},
            },
            {
                "data": {"key": [0, 0]},
                "attr": "key",
                "value": 1,
                "index": 1,
                "expected": {"key": [0, 1]},
            },
            {
                "data": {"key": [0]},
                "attr": "key",
                "value": Undefined,
                "index": 0,
                "expected": {"key": []},
            },
        ]
        invalid_cases = [
            {
                "data": {"key": 1},
                "attr": "key",
                "value": [1],
                "index": None,
                "expected": {},
            },
        ]
        for case in cases:
            data, attr, value, index, expected = (
                to_kcl_dict_obj(case["data"]),
                case["attr"],
                to_kcl_obj(case["value"]),
                case["index"],
                case["expected"],
            )
            data.list_key_override(attr, value, index)
            self.assertEqual(to_python_obj(data), expected)
        for case in invalid_cases:
            data, attr, value, index, expected = (
                to_kcl_dict_obj(case["data"]),
                case["attr"],
                to_kcl_obj(case["value"]),
                case["index"],
                case["expected"],
            )
            with self.assertRaises(kcl_error.KCLException):
                data.list_key_override(attr, value, index)

    def test_dict_object_insert_with(self):
        cases = [
            {"data": {}, "insert_data": {}, "index": None, "expected": {}},
            {"data": {}, "insert_data": None, "index": None, "expected": {}},
            {
                "data": {"key": []},
                "insert_data": {"key": [0]},
                "index": None,
                "expected": {"key": [0]},
            },
            {
                "data": {"key": [0]},
                "insert_data": {"key": [1]},
                "index": None,
                "expected": {"key": [0, 1]},
            },
            {
                "data": {"key": [0], "key_val": "val"},
                "insert_data": {"key": [1]},
                "index": None,
                "expected": {"key": [0, 1], "key_val": "val"},
            },
            {
                "data": {"key": [0]},
                "insert_data": {"key": [1]},
                "index": -1,
                "expected": {"key": [0, 1]},
            },
            {
                "data": {"key": [0]},
                "insert_data": {"key": [1]},
                "index": 1,
                "expected": {"key": [0, 1]},
            },
            {
                "data": {"key": [0]},
                "insert_data": {"key": [1]},
                "index": 0,
                "expected": {"key": [1, 0]},
            },
        ]
        for case in cases:
            data, insert_data, index, expected = (
                to_kcl_dict_obj(case["data"]),
                to_kcl_dict_obj(case["insert_data"]),
                case["index"],
                case["expected"],
            )
            data.insert_with(insert_data, index)
            self.assertEqual(to_python_obj(data), expected)

    def test_dict_object_has_key(self):
        cases = [
            {"data": {}, "key": None, "expected": False},
            {"data": {}, "key": "1", "expected": False},
            {"data": {}, "key": 1, "expected": False},
            {"data": {"key": "value"}, "key": "key_err", "expected": False},
            {"data": {"key": "value"}, "key": "key", "expected": True},
            {"data": {"key": {"key": "value"}}, "key": "key", "expected": True},
        ]
        for case in cases:
            data, key, expected = (
                to_kcl_dict_obj(case["data"]),
                case["key"],
                case["expected"],
            )
            self.assertEqual(data.has_key(key), expected)
            self.assertEqual(key in data, expected)

    def test_dict_object_get_key(self):
        cases = [
            {"data": {}, "key": None, "expected": Undefined},
            {"data": {}, "key": "1", "expected": Undefined},
            {"data": {}, "key": 1, "expected": Undefined},
            {"data": {"key": "value"}, "key": "key_err", "expected": Undefined},
            {"data": {"key": None}, "key": "key", "expected": None},
            {"data": {"key": "value"}, "key": "key", "expected": "value"},
            {
                "data": {"key": {"key": "value"}},
                "key": "key",
                "expected": {"key": "value"},
            },
        ]
        for case in cases:
            data, key, expected = (
                to_kcl_dict_obj(case["data"]),
                case["key"],
                case["expected"],
            )
            self.assertEqual(to_python_obj(data.get(key)), expected)

    def test_dict_object_update(self):
        cases = [
            {"data": {}, "update": {"key": "value"}, "expected": {"key": "value"}},
            {"data": {}, "update": {"key": 1}, "expected": {"key": 1}},
            {
                "data": {"key": "value"},
                "update": {"key": "override"},
                "expected": {"key": "override"},
            },
            {
                "data": {"key1": "value1"},
                "update": {"key2": "value2"},
                "expected": {"key1": "value1", "key2": "value2"},
            },
        ]
        for case in cases:
            data, update, expected = (
                to_kcl_dict_obj(case["data"]),
                case["update"],
                case["expected"],
            )
            data.update(update)
            self.assertEqual(to_python_obj(data), expected)

        for case in cases:
            data, update, expected = (
                to_kcl_dict_obj(case["data"]),
                to_kcl_obj(case["update"]),
                case["expected"],
            )
            data.update(update)
            self.assertEqual(to_python_obj(data), expected)

    def test_dict_unique_merge(self):
        cases = [
            {"data": {}, "update": {"key": "value"}, "expected": {"key": "value"}},
            {"data": {}, "update": {"key": 1}, "expected": {"key": 1}},
            {
                "data": {"key1": "value1"},
                "update": {"key2": "value2"},
                "expected": {"key1": "value1", "key2": "value2"},
            },
        ]
        for case in cases:
            data, update, expected = (
                to_kcl_dict_obj(case["data"]),
                to_kcl_obj(case["update"]),
                case["expected"],
            )
            data.unique_merge_with(update)
            self.assertEqual(to_python_obj(data), expected)

    def test_dict_delete(self):
        cases = [
            {"data": {"key": "value"}, "key": "key", "expected": {}},
            {
                "data": {"key1": "value1", "key2": "value2"},
                "key": "key1",
                "expected": {"key2": "value2"},
            },
        ]
        for case in cases:
            data, key, expected = (
                to_kcl_dict_obj(case["data"]),
                case["key"],
                case["expected"],
            )
            data.delete(key)
            self.assertEqual(to_python_obj(data), expected)


if __name__ == "__main__":
    unittest.main(verbosity=2)
