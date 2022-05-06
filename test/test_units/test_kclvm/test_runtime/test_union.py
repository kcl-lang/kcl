# Copyright 2020 The KCL Authors. All rights reserved.

import unittest

import kclvm.kcl.ast as ast
import kclvm.kcl.error as kcl_error
import kclvm.api.object as obj
import kclvm.vm as vm
from kclvm.vm.runtime.evaluator.union import union, resolve_schema_obj

VALUE1_KEY = "value1"
VALUE2_KEY = "value2"
EXPECTED_KEY = "expected"


class TestUnion(unittest.TestCase):
    def get_vm(self):
        app = obj.KCLProgram()
        app.pkgs = {"testpkg": obj.KCLBytecode()}
        app.main = "testpkg"
        test_vm = vm.VirtualMachine(app=app)
        test_f = vm.Frame()
        test_f.locals = {}
        test_f.globals = {}
        test_vm.ctx = test_f
        return test_vm

    def test_union(self):
        cases = [
            # Left None
            {VALUE1_KEY: None, VALUE2_KEY: 1, EXPECTED_KEY: 1},
            {VALUE1_KEY: None, VALUE2_KEY: 1.1, EXPECTED_KEY: 1.1},
            {VALUE1_KEY: None, VALUE2_KEY: [], EXPECTED_KEY: []},
            {VALUE1_KEY: None, VALUE2_KEY: {}, EXPECTED_KEY: {}},
            {VALUE1_KEY: None, VALUE2_KEY: "s", EXPECTED_KEY: "s"},
            {VALUE1_KEY: None, VALUE2_KEY: True, EXPECTED_KEY: True},
            {VALUE1_KEY: None, VALUE2_KEY: False, EXPECTED_KEY: False},
            {VALUE1_KEY: None, VALUE2_KEY: None, EXPECTED_KEY: None},
            # Right None
            {VALUE1_KEY: 1, VALUE2_KEY: None, EXPECTED_KEY: 1},
            {VALUE1_KEY: 1.1, VALUE2_KEY: None, EXPECTED_KEY: 1.1},
            {VALUE1_KEY: [], VALUE2_KEY: None, EXPECTED_KEY: []},
            {VALUE1_KEY: {}, VALUE2_KEY: None, EXPECTED_KEY: {}},
            {VALUE1_KEY: "s", VALUE2_KEY: None, EXPECTED_KEY: "s"},
            {VALUE1_KEY: True, VALUE2_KEY: None, EXPECTED_KEY: True},
            {VALUE1_KEY: False, VALUE2_KEY: None, EXPECTED_KEY: False},
            {VALUE1_KEY: None, VALUE2_KEY: None, EXPECTED_KEY: None},
            # Int
            {VALUE1_KEY: 1, VALUE2_KEY: 1, EXPECTED_KEY: 1},
            {VALUE1_KEY: 1, VALUE2_KEY: 2, EXPECTED_KEY: 2},
            {VALUE1_KEY: 1, VALUE2_KEY: 3, EXPECTED_KEY: 3},
            # Float
            {VALUE1_KEY: 1.0, VALUE2_KEY: 1.0, EXPECTED_KEY: 1.0},
            {VALUE1_KEY: 1.0, VALUE2_KEY: 1.5, EXPECTED_KEY: 1.5},
            # String
            {VALUE1_KEY: "s", VALUE2_KEY: "", EXPECTED_KEY: ""},
            {VALUE1_KEY: "s", VALUE2_KEY: "s", EXPECTED_KEY: "s"},
            {VALUE1_KEY: "s", VALUE2_KEY: "ss", EXPECTED_KEY: "ss"},
            # Boolean True
            {VALUE1_KEY: True, VALUE2_KEY: True, EXPECTED_KEY: True},
            {VALUE1_KEY: True, VALUE2_KEY: False, EXPECTED_KEY: False},
            # Boolean False
            {VALUE1_KEY: False, VALUE2_KEY: False, EXPECTED_KEY: False},
            {VALUE1_KEY: False, VALUE2_KEY: True, EXPECTED_KEY: True},
            # List
            {VALUE1_KEY: [], VALUE2_KEY: [], EXPECTED_KEY: []},
            {VALUE1_KEY: [], VALUE2_KEY: [1], EXPECTED_KEY: [1]},
            {VALUE1_KEY: [], VALUE2_KEY: [1, 2], EXPECTED_KEY: [1, 2]},
            {VALUE1_KEY: [1], VALUE2_KEY: [1], EXPECTED_KEY: [1]},
            {VALUE1_KEY: [1], VALUE2_KEY: [2], EXPECTED_KEY: [2]},
            {VALUE1_KEY: [1], VALUE2_KEY: [2, 2], EXPECTED_KEY: [2, 2]},
            {VALUE1_KEY: [1, 2], VALUE2_KEY: [3, 4], EXPECTED_KEY: [3, 4]},
            {VALUE1_KEY: [1, 2, 3], VALUE2_KEY: [3, 4], EXPECTED_KEY: [3, 4, 3]},
            {
                VALUE1_KEY: [{"key1": "value1"}],
                VALUE2_KEY: [{"key1": "value1"}],
                EXPECTED_KEY: [{"key1": "value1"}],
            },
            {
                VALUE1_KEY: [{"key1": "value1"}],
                VALUE2_KEY: [{"key1": "value2"}],
                EXPECTED_KEY: [{"key1": "value2"}],
            },
            {
                VALUE1_KEY: [{"key1": "value1"}],
                VALUE2_KEY: [{"key2": "value2"}],
                EXPECTED_KEY: [{"key1": "value1", "key2": "value2"}],
            },
            {
                VALUE1_KEY: [{"key1": "value1"}],
                VALUE2_KEY: [{"key1": "value1", "key2": "value2"}],
                EXPECTED_KEY: [{"key1": "value1", "key2": "value2"}],
            },
            {
                VALUE1_KEY: [{"key1": "value1"}],
                VALUE2_KEY: [{"key2": "value2", "key1": "value1"}],
                EXPECTED_KEY: [{"key1": "value1", "key2": "value2"}],
            },
            {
                VALUE1_KEY: [{"key1": "value1", "key2": "value2"}],
                VALUE2_KEY: [{"key1": "value1"}],
                EXPECTED_KEY: [{"key1": "value1", "key2": "value2"}],
            },
            # Dict
            {VALUE1_KEY: {}, VALUE2_KEY: {}, EXPECTED_KEY: {}},
            {
                VALUE1_KEY: {},
                VALUE2_KEY: {"key1": "value1"},
                EXPECTED_KEY: {"key1": "value1"},
            },
            {
                VALUE1_KEY: {"key1": "value1"},
                VALUE2_KEY: {},
                EXPECTED_KEY: {"key1": "value1"},
            },
            {
                VALUE1_KEY: {"key1": "value1"},
                VALUE2_KEY: {"key1": "value1"},
                EXPECTED_KEY: {"key1": "value1"},
            },
            {
                VALUE1_KEY: {"key1": "value1"},
                VALUE2_KEY: {"key1": "value1", "key2": "value2"},
                EXPECTED_KEY: {"key1": "value1", "key2": "value2"},
            },
            {
                VALUE1_KEY: {"key1": "value1", "key2": "value2"},
                VALUE2_KEY: {"key1": "value1"},
                EXPECTED_KEY: {"key1": "value1", "key2": "value2"},
            },
            {
                VALUE1_KEY: {"s": {"key1": "value1"}},
                VALUE2_KEY: {"s": {"key1": "value1", "key2": "value2"}},
                EXPECTED_KEY: {"s": {"key1": "value1", "key2": "value2"}},
            },
            {
                VALUE1_KEY: {"s": {"key1": "value1", "key2": "value2"}},
                VALUE2_KEY: {"s": {"key1": "value1"}},
                EXPECTED_KEY: {"s": {"key1": "value1", "key2": "value2"}},
            },
            {
                VALUE1_KEY: {"key2": "value2", "key1": "value1"},
                VALUE2_KEY: {"key1": "value1", "key2": "value2"},
                EXPECTED_KEY: {"key1": "value1", "key2": "value2"},
            },
            {
                VALUE1_KEY: {"key2": "value2", "key1": "value1"},
                VALUE2_KEY: {"key1": "value1", "key2": "value2"},
                EXPECTED_KEY: {"key1": "value1", "key2": "value2"},
            },
        ]
        for case in cases:
            try:
                value1 = obj.to_kcl_obj(case[VALUE1_KEY])
                value2 = obj.to_kcl_obj(case[VALUE2_KEY])
                union_value = obj.to_python_obj(union(value1, value2))
                expected = obj.to_python_obj(case[EXPECTED_KEY])
                self.assertEqual(union_value, expected)
            except AssertionError as err:
                print(
                    f"Assert fail between the value1 {value1} and the value2 {value2}"
                )
                raise err

    def test_union_with_idempotent_check(self):
        cases = [
            {
                VALUE1_KEY: obj.KCLSchemaConfigObject(
                    value=obj.to_kcl_obj({"key": [0]}).value,
                ),
                VALUE2_KEY: obj.KCLSchemaConfigObject(
                    value=obj.to_kcl_obj({"key": [1]}).value,
                    operation_map={"key": ast.ConfigEntryOperation.INSERT},
                    insert_index_map={"key": -1},
                ),
                EXPECTED_KEY: {"key": [0, 1]},
            },
            {
                VALUE1_KEY: obj.KCLSchemaConfigObject(
                    value=obj.to_kcl_obj({"key": [0]}).value,
                ),
                VALUE2_KEY: obj.KCLSchemaConfigObject(
                    value=obj.to_kcl_obj({"key": 1}).value,
                    operation_map={"key": ast.ConfigEntryOperation.INSERT},
                    insert_index_map={"key": 0},
                ),
                EXPECTED_KEY: {"key": [1, 0]},
            },
            {
                VALUE1_KEY: obj.KCLSchemaObject(
                    attrs=obj.to_kcl_obj({"key": [0]}).value,
                ),
                VALUE2_KEY: obj.KCLSchemaObject(
                    attrs=obj.to_kcl_obj({"key": 1}).value,
                    operation_map={"key": ast.ConfigEntryOperation.INSERT},
                    insert_index_map={"key": 0},
                ),
                EXPECTED_KEY: {"key": [1, 0]},
            },
            {
                VALUE1_KEY: obj.KCLSchemaObject(
                    attrs=obj.to_kcl_obj({"key": [0]}).value,
                ),
                VALUE2_KEY: obj.KCLSchemaObject(
                    attrs=obj.to_kcl_obj({"key": [1]}).value,
                    operation_map={"key": ast.ConfigEntryOperation.OVERRIDE},
                ),
                EXPECTED_KEY: {"key": [1]},
            },
            {
                VALUE1_KEY: obj.KCLSchemaObject(
                    attrs=obj.to_kcl_obj({"key": [0, 1]}).value,
                    operation_map={"key": ast.ConfigEntryOperation.OVERRIDE},
                ),
                VALUE2_KEY: obj.KCLSchemaObject(
                    attrs=obj.to_kcl_obj({"key": obj.Undefined}).value,
                    operation_map={"key": ast.ConfigEntryOperation.OVERRIDE},
                    insert_index_map={"key": 0},
                ),
                EXPECTED_KEY: {"key": [1]},
            },
        ]
        invalid_cases = [
            {
                VALUE1_KEY: {"key": 1},
                VALUE2_KEY: {"key": 2},
            },
            {
                VALUE1_KEY: {"key": 1.0},
                VALUE2_KEY: {"key": 2.0},
            },
            {
                VALUE1_KEY: {"key": True},
                VALUE2_KEY: {"key": False},
            },
            {
                VALUE1_KEY: obj.KCLSchemaConfigObject(
                    value=obj.to_kcl_obj({"key": [0]}).value,
                ),
                VALUE2_KEY: obj.KCLSchemaConfigObject(
                    value=obj.to_kcl_obj({"key": [1]}).value,
                ),
            },
            {
                VALUE1_KEY: obj.KCLSchemaObject(
                    attrs=obj.to_kcl_obj({"key": [0]}).value,
                ),
                VALUE2_KEY: obj.KCLSchemaObject(
                    attrs=obj.to_kcl_obj({"key": [1]}).value,
                ),
            },
        ]
        for case in cases:
            try:
                value1 = obj.to_kcl_obj(case[VALUE1_KEY])
                value2 = obj.to_kcl_obj(case[VALUE2_KEY])
                union_value = obj.to_python_obj(
                    union(value1, value2, should_idempotent_check=True)
                )
                expected = obj.to_python_obj(case[EXPECTED_KEY])
                self.assertEqual(union_value, expected)
            except AssertionError as err:
                print(
                    f"Assert fail between the value1 {value1} and the value2 {value2}"
                )
                raise err
        for case in invalid_cases:
            value1 = obj.to_kcl_obj(case[VALUE1_KEY])
            value2 = obj.to_kcl_obj(case[VALUE2_KEY])
            with self.assertRaises(kcl_error.KCLException):
                union(value1, value2, should_idempotent_check=True)

    def test_resolve_schema_obj(self):
        cases = [
            {"schema_obj": None, "keys": None, "vm": None, "expected": None},
            {
                "schema_obj": obj.KCLSchemaObject(name="Person"),
                "keys": set(),
                "vm": self.get_vm(),
                "expected": obj.KCLSchemaObject(name="Person"),
            },
        ]
        for case in cases:
            schema_obj, keys, vm, expected = (
                case["schema_obj"],
                case["keys"],
                case["vm"],
                case["expected"],
            )
            self.assertEqual(resolve_schema_obj(schema_obj, keys, vm), expected)


if __name__ == "__main__":
    unittest.main(verbosity=2)
