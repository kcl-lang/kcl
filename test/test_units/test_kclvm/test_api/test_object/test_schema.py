# Copyright 2020 The KCL Authors. All rights reserved.
import unittest

import kclvm.kcl.error as kcl_error
import kclvm.api.object.internal.decorators as decorators
from kclvm.api.object.object import (
    to_kcl_obj,
    to_python_obj,
    Undefined,
    KCLIntObject,
    KCLSchemaObject,
    KCLSchemaConfigObject,
    KCLStringLitTypeObject,
)
from kclvm.api.object.function import KCLCompiledFunctionObject, Parameter, KWArg
from kclvm.api.object.schema import KCLSchemaTypeObject
from kclvm.api.object.decorator import KCLDecoratorObject
from kclvm.kcl import ast


def build_test_schema_type_obj() -> KCLSchemaTypeObject:
    schema_type_obj = KCLSchemaTypeObject.new(
        "Person", None, "test.k", pkgpath="__main__", attr_list=["name", "age"]
    )
    schema_type_obj.set_func(
        KCLCompiledFunctionObject(
            name="Person",
            params=[
                Parameter(
                    name="name", value=to_kcl_obj("Alice"), type_annotation="str"
                ),
                Parameter(name="age", value=to_kcl_obj(18), type_annotation="int"),
                Parameter(
                    name="sex",
                    value=to_kcl_obj("Male"),
                    type_annotation='"Male"|"Female"',
                ),
            ],
        )
    )
    return schema_type_obj


def to_kcl_schema_obj(data: dict) -> KCLSchemaObject:
    return KCLSchemaObject(name="Person", attrs=to_kcl_obj(data).value)


class TestSchemaObject(unittest.TestCase):
    def test_dict_object_append_unpack(self):
        cases = [
            {
                "data": KCLSchemaObject(attrs={"key1": KCLIntObject(1)}),
                "item": {"key2": 2},
                "expected": {"key1": 1, "key2": 2},
            },
            {"data": KCLSchemaObject(attrs={"key1": KCLIntObject(1)}), "item": None, "expected": {"key1": 1}},
            {"data": KCLSchemaObject(attrs={"key1": KCLIntObject(1)}), "item": Undefined, "expected": {"key1": 1}},
            {
                "data": KCLSchemaObject(attrs={"key1": KCLIntObject(1)}),
                "item": KCLSchemaConfigObject(
                    value={"key1": KCLIntObject(2)},
                    operation_map={"key1": ast.ConfigEntryOperation.OVERRIDE},
                ),
                "expected": {"key1": 2},
            },
            {
                "data": KCLSchemaObject(attrs={"key1": KCLIntObject(1)}),
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

    def test_schema_object_update(self):
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
                to_kcl_schema_obj(case["data"]),
                case["update"],
                case["expected"],
            )
            data.update(update)
            self.assertEqual(to_python_obj(data), expected)

        for case in cases:
            data, update, expected = (
                to_kcl_schema_obj(case["data"]),
                case["update"],
                case["expected"],
            )
            for k, v in update.items():
                data.update_key_value(k, v)
            self.assertEqual(to_python_obj(data), expected)

    def test_schema_delete(self):
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
                to_kcl_schema_obj(case["data"]),
                case["key"],
                case["expected"],
            )
            data.delete(key)
            self.assertEqual(to_python_obj(data), expected)

    def test_schema_set_node_of_attr(self):
        schema_type_obj = build_test_schema_type_obj()
        node = ast.AST()
        schema_type_obj.set_node_of_attr("test_name", node)
        self.assertEqual(schema_type_obj.attr_obj_map["test_name"].attr_node, node)

        node_1 = ast.AST()
        schema_type_obj.set_node_of_attr("test_name", node_1)
        self.assertNotEqual(schema_type_obj.attr_obj_map["test_name"].attr_node, node)
        self.assertEqual(schema_type_obj.attr_obj_map["test_name"].attr_node, node_1)

    def test_schema_set_type_of_attr(self):
        schema_type_obj = build_test_schema_type_obj()
        tpe = KCLStringLitTypeObject()
        schema_type_obj.set_type_of_attr("test_name", tpe)
        self.assertEqual(schema_type_obj.attr_obj_map["test_name"].attr_type, tpe)

        tpe_1 = KCLStringLitTypeObject(value="test_tpe")
        schema_type_obj.set_type_of_attr("test_name", tpe_1)
        self.assertNotEqual(schema_type_obj.attr_obj_map["test_name"].attr_type, tpe)
        self.assertEqual(schema_type_obj.attr_obj_map["test_name"].attr_type, tpe_1)

    def test_schema_decorator(self):
        schema_obj = to_kcl_schema_obj({"key": "value"})
        schema_obj.add_decorator("key", KCLDecoratorObject(
            name="Deprecated",
            target=decorators.DecoratorTargetType.ATTRIBUTE,
            key="key",
            value="value",
            decorator=decorators.Deprecated,
        ))
        # Deprecated decorator will raise an error
        with self.assertRaises(Exception):
            schema_obj.run_all_decorators()


class TestSchemaArgsTypeCheck(unittest.TestCase):
    def test_schema_object_do_args_type_check_normal_only_args(self):
        schema_type_obj = build_test_schema_type_obj()
        cases = [
            [to_kcl_obj("Alice"), to_kcl_obj(18)],
            [to_kcl_obj("Bob"), to_kcl_obj(10)],
            [to_kcl_obj("John"), to_kcl_obj(10), to_kcl_obj("Female")],
        ]
        for case in cases:
            schema_type_obj.do_args_type_check(case, None, {})

    def test_schema_object_do_args_type_check_normal_only_kwargs(self):
        schema_type_obj = build_test_schema_type_obj()
        cases = [
            [
                KWArg(name=to_kcl_obj("name"), value=to_kcl_obj("Alice")),
                KWArg(name=to_kcl_obj("age"), value=to_kcl_obj(18)),
            ],
            [
                KWArg(name=to_kcl_obj("name"), value=to_kcl_obj("Bob")),
                KWArg(name=to_kcl_obj("age"), value=to_kcl_obj(10)),
            ],
            [
                KWArg(name=to_kcl_obj("sex"), value=to_kcl_obj("Male")),
                KWArg(name=to_kcl_obj("name"), value=to_kcl_obj("Bob")),
                KWArg(name=to_kcl_obj("age"), value=to_kcl_obj(10)),
            ],
        ]
        for case in cases:
            schema_type_obj.do_args_type_check(None, case, {})

    def test_schema_object_unexpected_keyword_argument(self):
        schema_type_obj = build_test_schema_type_obj()
        cases = [
            [
                KWArg(name=to_kcl_obj("err_name"), value=to_kcl_obj("Alice")),
                KWArg(name=to_kcl_obj("age"), value=to_kcl_obj(18)),
            ],
            [
                KWArg(name=to_kcl_obj("name"), value=to_kcl_obj("Bob")),
                KWArg(name=to_kcl_obj("err_age"), value=to_kcl_obj(10)),
            ],
            [
                KWArg(name=to_kcl_obj("err_sex"), value=to_kcl_obj("Male")),
                KWArg(name=to_kcl_obj("name"), value=to_kcl_obj("Bob")),
                KWArg(name=to_kcl_obj("age"), value=to_kcl_obj(10)),
            ],
        ]
        for case in cases:
            with self.assertRaises(kcl_error.EvaluationError) as err:
                schema_type_obj.do_args_type_check(None, case, {})
            self.assertEqual(
                err.exception.ewcode, kcl_error.ErrEwcode.EvaluationError_Ew
            )
            self.assertIn(
                "schema arguments got an unexpected keyword argument",
                str(err.exception),
            )


class TestSchemaTypeInstancesFunction(unittest.TestCase):
    def test_schema_type_instances(self):
        schema_type_obj = build_test_schema_type_obj()
        schema_type_obj.__refs__.append(
            KCLSchemaObject(name="Person", instance_pkgpath="__main__")
        )
        schema_type_obj.__refs__.append(
            KCLSchemaObject(name="Person", instance_pkgpath="pkg.to.path")
        )
        self.assertEqual(len(schema_type_obj.instances()), 1)
        self.assertEqual(len(schema_type_obj.instances(main_pkg=False)), 2)


if __name__ == "__main__":
    unittest.main(verbosity=2)
