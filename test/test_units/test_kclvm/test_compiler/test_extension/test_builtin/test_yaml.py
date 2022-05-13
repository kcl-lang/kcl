# Copyright 2020 The KCL Authors. All rights reserved.

import unittest

import kclvm.compiler.extension.builtin.system_module.yaml as yaml


class TestYAMLSystemModule(unittest.TestCase):
    def test_decode(self):
        yamlStrList = [
            "key: value",
            "- 1\n- 2\n- 3",
            "1",
            "1.1",
            "null",
            "true",
        ]
        expected = [
            {"key": "value"},
            [1, 2, 3],
            1,
            1.1,
            None,
            True,
        ]
        data = [yaml.KMANGLED_decode(s) for s in yamlStrList]
        self.assertListEqual(data, expected)

    def test_encode_literal(self):
        dataDict = {"key": "value"}
        dataList = [1, 2, 3]
        dataInt = 1
        dataFloat = 1.1
        dataNone = None
        dataBool = True
        expected = [
            "key: value\n",
            "- 1\n- 2\n- 3\n",
            "1\n...\n",
            "1.1\n...\n",
            "null\n...\n",
            "true\n...\n",
        ]
        yamlStr = [
            yaml.KMANGLED_encode(data)
            for data in [dataDict, dataList, dataInt, dataFloat, dataNone, dataBool]
        ]
        self.assertEqual(yamlStr, expected)

    def test_encode_dict(self):
        cases = [
            {
                "value": {
                    "key1": "value1",
                    "key2": "value2",
                    "data": [1, 2, 3],
                },
                "expected": """\
key1: value1
key2: value2
data:
- 1
- 2
- 3
""",
            },
            {
                "value": {
                    "key1": {"key1": "value1", "key2": "value2"},
                    "key2": "value2",
                    "data": [1, 2, 3],
                },
                "expected": """\
key1:
  key1: value1
  key2: value2
key2: value2
data:
- 1
- 2
- 3
""",
            },
        ]
        for case in cases:
            value, expected = case["value"], case["expected"]
            self.assertEqual(yaml.KMANGLED_encode(value), expected)

    def test_encode_dict(self):
        cases = [
            {
                "value": {
                    "key1": "value1",
                    "key2": "value2",
                    "data": [1, 2, 3],
                },
                "expected": """\
key1: value1
key2: value2
data:
- 1
- 2
- 3
""",
            },
            {
                "value": {
                    "key1": {"key1": "value1", "key2": "value2"},
                    "key2": "value2",
                    "data": [1, 2, 3],
                },
                "expected": """\
key1:
  key1: value1
  key2: value2
key2: value2
data:
- 1
- 2
- 3
""",
            },
        ]
        for case in cases:
            value, expected = case["value"], case["expected"]
            self.assertEqual(yaml.KMANGLED_encode(value), expected)

    def test_encode_with_ignore_none(self):
        cases = [
            {
                "value": {
                    "key1": None,
                    "key2": "value2",
                    "data": [1, 2, 3],
                },
                "expected": """\
key2: value2
data:
- 1
- 2
- 3
""",
            },
            {
                "value": {
                    "key1": {"key1": "value1", "key2": "value2"},
                    "key2": None,
                    "data": [1, 2, 3],
                },
                "expected": """\
key1:
  key1: value1
  key2: value2
data:
- 1
- 2
- 3
""",
            },
        ]
        for case in cases:
            value, expected = case["value"], case["expected"]
            self.assertEqual(yaml.KMANGLED_encode(value, ignore_none=True), expected)

    def test_encode_with_ignore_private(self):
        cases = [
            {
                "value": {
                    "_key1": "value1",
                    "key2": "value2",
                    "data": [1, 2, 3],
                },
                "expected": """\
key2: value2
data:
- 1
- 2
- 3
""",
            },
            {
                "value": {
                    "key1": {"key1": "value1", "key2": "value2"},
                    "_key2": "value2",
                    "data": [1, 2, 3],
                },
                "expected": """\
key1:
  key1: value1
  key2: value2
data:
- 1
- 2
- 3
""",
            },
        ]
        for case in cases:
            value, expected = case["value"], case["expected"]
            self.assertEqual(yaml.KMANGLED_encode(value, ignore_private=True), expected)

    def test_encode_with_sort_keys(self):
        cases = [
            {
                "value": {
                    "key1": "value1",
                    "key2": "value2",
                    "data": [1, 2, 3],
                },
                "expected": """\
data:
- 1
- 2
- 3
key1: value1
key2: value2
""",
            },
            {
                "value": {
                    "key1": {"key1": "value1", "key2": "value2"},
                    "key2": "value2",
                    "data": [1, 2, 3],
                },
                "expected": """\
data:
- 1
- 2
- 3
key1:
  key1: value1
  key2: value2
key2: value2
""",
            },
        ]
        for case in cases:
            value, expected = case["value"], case["expected"]
            self.assertEqual(yaml.KMANGLED_encode(value, sort_keys=True), expected)

if __name__ == "__main__":
    unittest.main(verbosity=2)
