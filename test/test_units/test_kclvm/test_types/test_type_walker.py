# Copyright 2021 The KCL Authors. All rights reserved.

import unittest
import pathlib
from typing import Tuple

import kclvm.api.object as objpkg
import kclvm.kcl.types as types
import kclvm.kcl.types.walker as type_walker


class TypeTest(unittest.TestCase):
    def test_type_walker_convert_int_to_str(self):
        def walk_fn(t):
            if isinstance(t, objpkg.KCLIntTypeObject):
                return objpkg.KCLStringTypeObject()
            return t

        cases = [
            {"type": types.INT_TYPE, "expected": types.STR_TYPE},
            {
                "type": objpkg.KCLListTypeObject(types.ANY_TYPE),
                "expected": objpkg.KCLListTypeObject(types.ANY_TYPE),
            },
            {
                "type": objpkg.KCLListTypeObject(types.STR_TYPE),
                "expected": objpkg.KCLListTypeObject(types.STR_TYPE),
            },
            {
                "type": objpkg.KCLListTypeObject(types.INT_TYPE),
                "expected": objpkg.KCLListTypeObject(types.STR_TYPE),
            },
            {
                "type": objpkg.KCLUnionTypeObject([types.STR_TYPE, types.INT_TYPE]),
                "expected": objpkg.KCLUnionTypeObject(
                    types=[types.STR_TYPE, types.STR_TYPE]
                ),
            },
            {
                "type": objpkg.KCLUnionTypeObject(
                    [
                        types.STR_TYPE,
                        types.INT_TYPE,
                        objpkg.KCLUnionTypeObject(
                            types=[types.INT_TYPE, types.STR_TYPE]
                        ),
                    ]
                ),
                "expected": objpkg.KCLUnionTypeObject(
                    [
                        types.STR_TYPE,
                        types.STR_TYPE,
                        objpkg.KCLUnionTypeObject(
                            types=[types.STR_TYPE, types.STR_TYPE]
                        ),
                    ]
                ),
            },
            {
                "type": objpkg.KCLUnionTypeObject(
                    [types.BOOL_TYPE, types.TRUE_LIT_TYPE]
                ),
                "expected": objpkg.KCLUnionTypeObject(
                    [types.BOOL_TYPE, types.TRUE_LIT_TYPE]
                ),
            },
            {
                "type": objpkg.KCLDictTypeObject(
                    key_type=types.INT_TYPE, value_type=types.INT_TYPE
                ),
                "expected": objpkg.KCLDictTypeObject(
                    key_type=types.STR_TYPE, value_type=types.STR_TYPE
                ),
            },
        ]
        for case in cases:
            tpe, expected = case["type"], case["expected"]
            self.assertEqual(
                type_walker.WalkType(tpe, walk_fn),
                expected,
                msg=f"assert error on type {tpe}, and the expected is {expected}",
            )


if __name__ == "__main__":
    unittest.main(verbosity=2)
