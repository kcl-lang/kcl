# Copyright 2021 The KCL Authors. All rights reserved.

import unittest
import pathlib

import kclvm.kcl.error as kcl_error
import kclvm.api.object as objpkg
import kclvm.kcl.types as types


class TypeConvensionTest(unittest.TestCase):
    def test_type_convert(self):
        cases = [
            {
                "obj": objpkg.NONE_INSTANCE,
                "tpe": objpkg.KCLIntTypeObject(),
                "expected": objpkg.NONE_INSTANCE,
            },
            {
                "obj": objpkg.UNDEFINED_INSTANCE,
                "tpe": objpkg.KCLIntTypeObject(),
                "expected": objpkg.UNDEFINED_INSTANCE,
            },
            {
                "obj": objpkg.KCLIntObject(1),
                "tpe": types.ANY_TYPE,
                "expected": objpkg.KCLIntObject(1),
            },
            {
                "obj": objpkg.KCLDictObject(value={"key": objpkg.KCLStringObject("s")}),
                "tpe": types.DICT_STR_STR_TYPE,
                "expected": objpkg.KCLDictObject(value={"key": objpkg.KCLStringObject("s")}),
            },
            {
                "obj": objpkg.KCLListObject(items=[objpkg.KCLStringObject("s")]),
                "tpe": objpkg.KCLListTypeObject(objpkg.KCLStringTypeObject()),
                "expected": objpkg.KCLListObject(items=[objpkg.KCLStringObject("s")]),
            },
        ]
        for case in cases:
            obj, tpe, expected = case["obj"], case["tpe"], case["expected"]
            self.assertEqual(types.type_convert(obj, tpe), expected)

    def test_type_convert_failed(self):
        cases = [
            {"obj": objpkg.KCLIntObject(0), "tpe": objpkg.KCLStringTypeObject()},
            {"obj": objpkg.KCLStringObject("s"), "tpe": objpkg.KCLIntTypeObject()},
        ]
        for case in cases:
            obj, tpe = case["obj"], case["tpe"]
            with self.assertRaises(kcl_error.EvaluationError):
                types.type_convert(obj, tpe)

    def test_type_convert_invalid_params(self):
        cases = [
            {"obj": None, "tpe": None},
            {"obj": objpkg.KCLIntObject(0), "tpe": None},
            {"obj": None, "tpe": objpkg.KCLIntTypeObject()},
        ]
        for case in cases:
            obj, tpe = case["obj"], case["tpe"]
            with self.assertRaises(ValueError):
                types.type_convert(obj, tpe)


if __name__ == "__main__":
    unittest.main(verbosity=2)
