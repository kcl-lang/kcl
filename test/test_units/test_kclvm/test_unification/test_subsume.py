#! /usr/bin/env python3

import os
import unittest
import pathlib
from typing import Tuple

import kclvm.api.object as obj
import kclvm.kcl.types as types
from kclvm.compiler.parser import ParseFile
from kclvm.unification import value_subsume, type_subsume

VALUE1_KEY = "value1"
VALUE2_KEY = "value2"
EXPECTED_KEY = "expected"


class KCLSubsumeTest(unittest.TestCase):
    def test_value_subsume(self):
        cases = [
            # Left None
            {VALUE1_KEY: None, VALUE2_KEY: 1, EXPECTED_KEY: True},
            {VALUE1_KEY: None, VALUE2_KEY: 1.1, EXPECTED_KEY: True},
            {VALUE1_KEY: None, VALUE2_KEY: [], EXPECTED_KEY: True},
            {VALUE1_KEY: None, VALUE2_KEY: {}, EXPECTED_KEY: True},
            {VALUE1_KEY: None, VALUE2_KEY: "s", EXPECTED_KEY: True},
            {VALUE1_KEY: None, VALUE2_KEY: True, EXPECTED_KEY: True},
            {VALUE1_KEY: None, VALUE2_KEY: False, EXPECTED_KEY: True},
            {VALUE1_KEY: None, VALUE2_KEY: None, EXPECTED_KEY: True},
            # Right None
            {VALUE1_KEY: 1, VALUE2_KEY: None, EXPECTED_KEY: True},
            {VALUE1_KEY: 1.1, VALUE2_KEY: None, EXPECTED_KEY: True},
            {VALUE1_KEY: [], VALUE2_KEY: None, EXPECTED_KEY: True},
            {VALUE1_KEY: {}, VALUE2_KEY: None, EXPECTED_KEY: True},
            {VALUE1_KEY: "s", VALUE2_KEY: None, EXPECTED_KEY: True},
            {VALUE1_KEY: True, VALUE2_KEY: None, EXPECTED_KEY: True},
            {VALUE1_KEY: False, VALUE2_KEY: None, EXPECTED_KEY: True},
            {VALUE1_KEY: None, VALUE2_KEY: None, EXPECTED_KEY: True},
            # Left Undefined
            {VALUE1_KEY: obj.Undefined, VALUE2_KEY: 1, EXPECTED_KEY: True},
            {VALUE1_KEY: obj.Undefined, VALUE2_KEY: 1.1, EXPECTED_KEY: True},
            {VALUE1_KEY: obj.Undefined, VALUE2_KEY: [], EXPECTED_KEY: True},
            {VALUE1_KEY: obj.Undefined, VALUE2_KEY: {}, EXPECTED_KEY: True},
            {VALUE1_KEY: obj.Undefined, VALUE2_KEY: "s", EXPECTED_KEY: True},
            {VALUE1_KEY: obj.Undefined, VALUE2_KEY: True, EXPECTED_KEY: True},
            {VALUE1_KEY: obj.Undefined, VALUE2_KEY: False, EXPECTED_KEY: True},
            {VALUE1_KEY: obj.Undefined, VALUE2_KEY: None, EXPECTED_KEY: True},
            # Right Undefined
            {VALUE1_KEY: 1, VALUE2_KEY: obj.Undefined, EXPECTED_KEY: True},
            {VALUE1_KEY: 1.1, VALUE2_KEY: obj.Undefined, EXPECTED_KEY: True},
            {VALUE1_KEY: [], VALUE2_KEY: obj.Undefined, EXPECTED_KEY: True},
            {VALUE1_KEY: {}, VALUE2_KEY: obj.Undefined, EXPECTED_KEY: True},
            {VALUE1_KEY: "s", VALUE2_KEY: obj.Undefined, EXPECTED_KEY: True},
            {VALUE1_KEY: True, VALUE2_KEY: obj.Undefined, EXPECTED_KEY: True},
            {VALUE1_KEY: False, VALUE2_KEY: obj.Undefined, EXPECTED_KEY: True},
            {VALUE1_KEY: None, VALUE2_KEY: obj.Undefined, EXPECTED_KEY: True},
            # Int
            {VALUE1_KEY: 1, VALUE2_KEY: 1, EXPECTED_KEY: True},
            {VALUE1_KEY: 1, VALUE2_KEY: 2, EXPECTED_KEY: False},
            {VALUE1_KEY: 1, VALUE2_KEY: 1.0, EXPECTED_KEY: False},
            {VALUE1_KEY: 1, VALUE2_KEY: 1.1, EXPECTED_KEY: False},
            {VALUE1_KEY: 1, VALUE2_KEY: [], EXPECTED_KEY: False},
            {VALUE1_KEY: 1, VALUE2_KEY: {}, EXPECTED_KEY: False},
            {VALUE1_KEY: 1, VALUE2_KEY: "s", EXPECTED_KEY: False},
            {VALUE1_KEY: 1, VALUE2_KEY: True, EXPECTED_KEY: False},
            {VALUE1_KEY: 1, VALUE2_KEY: False, EXPECTED_KEY: False},
            # Float
            {VALUE1_KEY: 1.1, VALUE2_KEY: 1, EXPECTED_KEY: False},
            {VALUE1_KEY: 1.1, VALUE2_KEY: 2, EXPECTED_KEY: False},
            {VALUE1_KEY: 1.1, VALUE2_KEY: 1.0, EXPECTED_KEY: False},
            {VALUE1_KEY: 1.1, VALUE2_KEY: 1.1, EXPECTED_KEY: True},
            {VALUE1_KEY: 1.1, VALUE2_KEY: [], EXPECTED_KEY: False},
            {VALUE1_KEY: 1.1, VALUE2_KEY: {}, EXPECTED_KEY: False},
            {VALUE1_KEY: 1.1, VALUE2_KEY: "s", EXPECTED_KEY: False},
            {VALUE1_KEY: 1.1, VALUE2_KEY: True, EXPECTED_KEY: False},
            {VALUE1_KEY: 1.1, VALUE2_KEY: False, EXPECTED_KEY: False},
            # String
            {VALUE1_KEY: "s", VALUE2_KEY: 1, EXPECTED_KEY: False},
            {VALUE1_KEY: "s", VALUE2_KEY: 2, EXPECTED_KEY: False},
            {VALUE1_KEY: "s", VALUE2_KEY: 1.0, EXPECTED_KEY: False},
            {VALUE1_KEY: "s", VALUE2_KEY: 1.1, EXPECTED_KEY: False},
            {VALUE1_KEY: "s", VALUE2_KEY: [], EXPECTED_KEY: False},
            {VALUE1_KEY: "s", VALUE2_KEY: {}, EXPECTED_KEY: False},
            {VALUE1_KEY: "s", VALUE2_KEY: "s", EXPECTED_KEY: True},
            {VALUE1_KEY: "s", VALUE2_KEY: "", EXPECTED_KEY: False},
            {VALUE1_KEY: "s", VALUE2_KEY: "ss", EXPECTED_KEY: False},
            {VALUE1_KEY: "s", VALUE2_KEY: True, EXPECTED_KEY: False},
            {VALUE1_KEY: "s", VALUE2_KEY: False, EXPECTED_KEY: False},
            # Boolean True
            {VALUE1_KEY: True, VALUE2_KEY: 1, EXPECTED_KEY: False},
            {VALUE1_KEY: True, VALUE2_KEY: 2, EXPECTED_KEY: False},
            {VALUE1_KEY: True, VALUE2_KEY: 1.0, EXPECTED_KEY: False},
            {VALUE1_KEY: True, VALUE2_KEY: 1.1, EXPECTED_KEY: False},
            {VALUE1_KEY: True, VALUE2_KEY: [], EXPECTED_KEY: False},
            {VALUE1_KEY: True, VALUE2_KEY: {}, EXPECTED_KEY: False},
            {VALUE1_KEY: True, VALUE2_KEY: "s", EXPECTED_KEY: False},
            {VALUE1_KEY: True, VALUE2_KEY: "", EXPECTED_KEY: False},
            {VALUE1_KEY: True, VALUE2_KEY: "ss", EXPECTED_KEY: False},
            {VALUE1_KEY: True, VALUE2_KEY: True, EXPECTED_KEY: True},
            {VALUE1_KEY: True, VALUE2_KEY: False, EXPECTED_KEY: False},
            # Boolean False
            {VALUE1_KEY: False, VALUE2_KEY: 1, EXPECTED_KEY: False},
            {VALUE1_KEY: False, VALUE2_KEY: 2, EXPECTED_KEY: False},
            {VALUE1_KEY: False, VALUE2_KEY: 1.0, EXPECTED_KEY: False},
            {VALUE1_KEY: False, VALUE2_KEY: 1.1, EXPECTED_KEY: False},
            {VALUE1_KEY: False, VALUE2_KEY: [], EXPECTED_KEY: False},
            {VALUE1_KEY: False, VALUE2_KEY: {}, EXPECTED_KEY: False},
            {VALUE1_KEY: False, VALUE2_KEY: "s", EXPECTED_KEY: False},
            {VALUE1_KEY: False, VALUE2_KEY: "", EXPECTED_KEY: False},
            {VALUE1_KEY: False, VALUE2_KEY: "ss", EXPECTED_KEY: False},
            {VALUE1_KEY: False, VALUE2_KEY: True, EXPECTED_KEY: False},
            {VALUE1_KEY: False, VALUE2_KEY: False, EXPECTED_KEY: True},
            # List
            {VALUE1_KEY: [], VALUE2_KEY: 1, EXPECTED_KEY: False},
            {VALUE1_KEY: [], VALUE2_KEY: 2, EXPECTED_KEY: False},
            {VALUE1_KEY: [], VALUE2_KEY: 1.0, EXPECTED_KEY: False},
            {VALUE1_KEY: [], VALUE2_KEY: 1.1, EXPECTED_KEY: False},
            {VALUE1_KEY: [], VALUE2_KEY: [], EXPECTED_KEY: True},
            {VALUE1_KEY: [], VALUE2_KEY: {}, EXPECTED_KEY: False},
            {VALUE1_KEY: [], VALUE2_KEY: "s", EXPECTED_KEY: False},
            {VALUE1_KEY: [], VALUE2_KEY: True, EXPECTED_KEY: False},
            {VALUE1_KEY: [], VALUE2_KEY: False, EXPECTED_KEY: False},
            {VALUE1_KEY: [1], VALUE2_KEY: [1], EXPECTED_KEY: True},
            {VALUE1_KEY: [1], VALUE2_KEY: [2], EXPECTED_KEY: False},
            {VALUE1_KEY: [1], VALUE2_KEY: [1, 1], EXPECTED_KEY: False},
            {
                VALUE1_KEY: [{"key1": "value1"}],
                VALUE2_KEY: [{"key1": "value1"}],
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: [{"key1": "value1"}],
                VALUE2_KEY: [{"key1": "value2"}],
                EXPECTED_KEY: False,
            },
            {
                VALUE1_KEY: [{"key1": "value1"}],
                VALUE2_KEY: [{"key2": "value2"}],
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: [{"key1": "value1"}],
                VALUE2_KEY: [{"key1": "value1", "key2": "value2"}],
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: [{"key1": "value1"}],
                VALUE2_KEY: [{"key2": "value2", "key1": "value1"}],
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: [{"key1": "value1", "key2": "value2"}],
                VALUE2_KEY: [{"key1": "value1"}],
                EXPECTED_KEY: True,
            },
            # Dict
            {VALUE1_KEY: {}, VALUE2_KEY: 1, EXPECTED_KEY: False},
            {VALUE1_KEY: {}, VALUE2_KEY: 2, EXPECTED_KEY: False},
            {VALUE1_KEY: {}, VALUE2_KEY: 1.0, EXPECTED_KEY: False},
            {VALUE1_KEY: {}, VALUE2_KEY: 1.1, EXPECTED_KEY: False},
            {VALUE1_KEY: {}, VALUE2_KEY: [], EXPECTED_KEY: False},
            {VALUE1_KEY: {}, VALUE2_KEY: {}, EXPECTED_KEY: True},
            {VALUE1_KEY: {}, VALUE2_KEY: "s", EXPECTED_KEY: False},
            {VALUE1_KEY: {}, VALUE2_KEY: True, EXPECTED_KEY: False},
            {VALUE1_KEY: {}, VALUE2_KEY: False, EXPECTED_KEY: False},
            {VALUE1_KEY: {}, VALUE2_KEY: {"key1": "value1"}, EXPECTED_KEY: True},
            {VALUE1_KEY: {"key1": "value1"}, VALUE2_KEY: {}, EXPECTED_KEY: True},
            {
                VALUE1_KEY: {"key1": "value1"},
                VALUE2_KEY: {"key1": "value1"},
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: {"key1": "value1"},
                VALUE2_KEY: {"key1": "value1", "key2": "value2"},
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: {"key1": "value1", "key2": "value2"},
                VALUE2_KEY: {"key1": "value1"},
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: {"s": {"key1": "value1"}},
                VALUE2_KEY: {"s": {"key1": "value1", "key2": "value2"}},
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: {"s": {"key1": "value1", "key2": "value2"}},
                VALUE2_KEY: {"s": {"key1": "value1"}},
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: {"key2": "value2", "key1": "value1"},
                VALUE2_KEY: {"key1": "value1", "key2": "value2"},
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: {"key2": "value2", "key1": "value1"},
                VALUE2_KEY: {"key1": "value1", "key2": "value2"},
                EXPECTED_KEY: True,
            },
            # Schema
        ]
        for case in cases:
            try:
                value1 = obj.to_kcl_obj(case[VALUE1_KEY])
                value2 = obj.to_kcl_obj(case[VALUE2_KEY])
                expected = case[EXPECTED_KEY]
                self.assertEqual(value_subsume(value1, value2), expected)
            except AssertionError as err:
                print(
                    f"Assert fail between the value1 {obj.to_python_obj(value1)} and the value2 {obj.to_python_obj(value2)}"
                )
                raise err

    def test_type_subsume(self):
        cases = [
            # The same types
            {VALUE1_KEY: None, VALUE2_KEY: None, EXPECTED_KEY: False},
            {VALUE1_KEY: 1, VALUE2_KEY: 2, EXPECTED_KEY: False},
            {
                VALUE1_KEY: obj.NONE_INSTANCE,
                VALUE2_KEY: obj.NONE_INSTANCE,
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: types.ANY_TYPE,
                VALUE2_KEY: types.ANY_TYPE,
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: types.STR_TYPE,
                VALUE2_KEY: types.STR_TYPE,
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: types.INT_TYPE,
                VALUE2_KEY: types.INT_TYPE,
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: types.FLOAT_TYPE,
                VALUE2_KEY: types.FLOAT_TYPE,
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: types.BOOL_TYPE,
                VALUE2_KEY: types.BOOL_TYPE,
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: types.NONE_TYPE,
                VALUE2_KEY: types.BOOL_TYPE,
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: types.INT_TYPE,
                VALUE2_KEY: types.FLOAT_TYPE,
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: types.DICT_STR_ANY_TYPE,
                VALUE2_KEY: types.DICT_STR_ANY_TYPE,
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: types.DICT_STR_STR_TYPE,
                VALUE2_KEY: types.DICT_STR_ANY_TYPE,
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: obj.KCLIntLitTypeObject(1),
                VALUE2_KEY: types.INT_TYPE,
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: obj.KCLFloatLitTypeObject(1.0),
                VALUE2_KEY: types.FLOAT_TYPE,
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: obj.KCLStringLitTypeObject("s"),
                VALUE2_KEY: types.STR_TYPE,
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: types.TRUE_LIT_TYPE,
                VALUE2_KEY: types.BOOL_TYPE,
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: types.INT_TYPE,
                VALUE2_KEY: obj.KCLUnionTypeObject([types.INT_TYPE, types.STR_TYPE]),
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: types.STR_TYPE,
                VALUE2_KEY: obj.KCLUnionTypeObject([types.INT_TYPE, types.STR_TYPE]),
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: types.INT_OR_STR_TYPE,
                VALUE2_KEY: obj.KCLUnionTypeObject([types.STR_TYPE, types.INT_TYPE]),
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: obj.KCLIntObject(1),
                VALUE2_KEY: obj.KCLUnionTypeObject([types.STR_TYPE, types.INT_TYPE]),
                EXPECTED_KEY: False,
            },
            {
                VALUE1_KEY: obj.KCLIntLitTypeObject(1),
                VALUE2_KEY: obj.KCLUnionTypeObject(
                    types=[
                        obj.KCLIntLitTypeObject(1),
                        types.TRUE_LIT_TYPE,
                        obj.KCLStringLitTypeObject("aaa"),
                    ]
                ),
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: obj.KCLNumberMultiplierTypeObject(
                    value=1024,
                    raw_value=1,
                    binary_suffix="1Mi"
                ),
                VALUE2_KEY: obj.KCLNumberMultiplierTypeObject(
                    value=1024,
                    raw_value=1,
                    binary_suffix="1Mi"
                ),
                EXPECTED_KEY: True,
            },
            {
                VALUE1_KEY: obj.KCLNumberMultiplierTypeObject(),
                VALUE2_KEY: obj.KCLNumberMultiplierTypeObject(
                    value=1024,
                    raw_value=1,
                    binary_suffix="1Mi"
                ),
                EXPECTED_KEY: True,
            },
        ]
        for case in cases:
            type1, type2, expected = (
                case[VALUE1_KEY],
                case[VALUE2_KEY],
                case[EXPECTED_KEY],
            )
            self.assertEqual(
                type_subsume(type1, type2),
                expected,
                msg=f"Assert error between {type1} and {type2}",
            )


if __name__ == "__main__":
    unittest.main(verbosity=2)
