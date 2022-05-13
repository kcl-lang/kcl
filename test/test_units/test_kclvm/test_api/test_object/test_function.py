# Copyright 2020 The KCL Authors. All rights reserved.

import pathlib
import unittest

import kclvm.kcl.error as kcl_error
import kclvm.api.object as objpkg


class TestFunctionObject(unittest.TestCase):
    def test_function_object(self):
        empty_func_obj = objpkg.KCLFunctionObject(name="fake")
        self.assertEqual(empty_func_obj.type(), objpkg.KCLObjectType.FUNCTION)
        self.assertEqual(empty_func_obj.type_str(), "function")
        self.assertEqual(empty_func_obj.call([], []), None)

    def test_closure_object(self):
        empty_func_obj = objpkg.KCLClosureObject(name="fake")
        self.assertEqual(empty_func_obj.type(), objpkg.KCLObjectType.CLOSURE)
        self.assertEqual(empty_func_obj.type_str(), "closure")

    def test_builtin_function_object(self):
        print_builtin_func_obj = objpkg.KCLBuiltinFunctionObject(
            name="print",
            function=print,
        )
        self.assertEqual(print_builtin_func_obj.call([], []), objpkg.NONE_INSTANCE)
        sum_builtin_func_obj = objpkg.KCLBuiltinFunctionObject(
            name="sum",
            function=sum,
        )
        self.assertEqual(
            sum_builtin_func_obj.call([objpkg.to_kcl_obj([1, 2, 3])], []),
            objpkg.to_kcl_obj(6),
        )
        invalid_builtin_func_obj = objpkg.KCLBuiltinFunctionObject(
            name="sum",
            function=None,
        )
        with self.assertRaises(Exception):
            invalid_builtin_func_obj.call([], [])


if __name__ == "__main__":
    unittest.main(verbosity=2)
