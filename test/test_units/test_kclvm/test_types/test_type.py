# Copyright 2021 The KCL Authors. All rights reserved.

import unittest
import pathlib
from typing import Tuple

import kclvm.api.object as objpkg
import kclvm.kcl.types as types


class TypeTest(unittest.TestCase):
    def test_sup(self):
        cases = [
            {"types": [], "expected": types.ANY_TYPE},
            {"types": [types.ANY_TYPE], "expected": types.ANY_TYPE},
            {"types": [types.STR_TYPE], "expected": types.STR_TYPE},
            {
                "types": [types.STR_TYPE, types.INT_TYPE],
                "expected": objpkg.KCLUnionTypeObject(
                    types=[types.INT_TYPE, types.STR_TYPE]
                ),
            },
            {
                "types": [
                    types.STR_TYPE,
                    types.INT_TYPE,
                    objpkg.KCLUnionTypeObject(types=[types.INT_TYPE, types.STR_TYPE]),
                ],
                "expected": objpkg.KCLUnionTypeObject(
                    types=[types.INT_TYPE, types.STR_TYPE]
                ),
            },
            {
                "types": [types.BOOL_TYPE, types.TRUE_LIT_TYPE],
                "expected": types.BOOL_TYPE,
            },
            {
                "types": [
                    objpkg.KCLStringLitTypeObject("Blue"),
                    objpkg.KCLStringLitTypeObject("Yellow"),
                    objpkg.KCLStringLitTypeObject("Red"),
                ],
                "expected": objpkg.KCLUnionTypeObject(
                    types=[
                        objpkg.KCLStringLitTypeObject("Blue"),
                        objpkg.KCLStringLitTypeObject("Yellow"),
                        objpkg.KCLStringLitTypeObject("Red"),
                    ]
                ),
            },
            {
                "types": [
                    objpkg.KCLListTypeObject(
                        objpkg.KCLUnionTypeObject(
                            [
                                objpkg.KCLIntLitTypeObject(1),
                                objpkg.KCLIntLitTypeObject(2),
                            ]
                        )
                    ),
                    objpkg.KCLListTypeObject(
                        objpkg.KCLUnionTypeObject(
                            [
                                objpkg.KCLIntLitTypeObject(3),
                                objpkg.KCLIntLitTypeObject(4),
                            ]
                        )
                    ),
                ],
                "expected": objpkg.KCLUnionTypeObject(
                    [
                        objpkg.KCLListTypeObject(
                            objpkg.KCLUnionTypeObject(
                                [
                                    objpkg.KCLIntLitTypeObject(1),
                                    objpkg.KCLIntLitTypeObject(2),
                                ]
                            )
                        ),
                        objpkg.KCLListTypeObject(
                            objpkg.KCLUnionTypeObject(
                                [
                                    objpkg.KCLIntLitTypeObject(3),
                                    objpkg.KCLIntLitTypeObject(4),
                                ]
                            ),
                        ),
                    ]
                ),
            },
            {
                "types": [
                    objpkg.KCLUnionTypeObject(
                        [
                            types.STR_TYPE,
                            types.DICT_STR_STR_TYPE,
                        ]
                    ),
                    types.DICT_ANY_ANY_TYPE,
                ],
                "expected": objpkg.KCLUnionTypeObject(
                    [
                        types.STR_TYPE,
                        types.DICT_ANY_ANY_TYPE,
                    ]
                ),
            },
        ]
        for case in cases:
            type_list, expected = case["types"], case["expected"]
            got = types.sup(type_list)
            self.assertEqual(
                got,
                expected,
                msg=f"assert error on type list {type_list}, got {got}",
            )

    def test_assignale_to(self):
        cases = [
            {"type1": types.NONE_TYPE, "type2": types.ANY_TYPE, "expected": True},
            {"type1": types.ANY_TYPE, "type2": types.ANY_TYPE, "expected": True},
            {"type1": types.ANY_TYPE, "type2": types.INT_TYPE, "expected": True},
            {"type1": types.INT_TYPE, "type2": types.ANY_TYPE, "expected": True},
            {"type1": types.INT_TYPE, "type2": types.FLOAT_TYPE, "expected": True},
            {
                "type1": objpkg.KCLStringLitTypeObject("ss"),
                "type2": types.STR_TYPE,
                "expected": True,
            },
            {
                "type1": types.INT_TYPE,
                "type2": objpkg.KCLUnionTypeObject(
                    types=[types.INT_TYPE, types.STR_TYPE]
                ),
                "expected": True,
            },
            {
                "type1": types.DICT_STR_STR_TYPE,
                "type2": types.DICT_STR_ANY_TYPE,
                "expected": True,
            },
            {"type1": types.VOID_TYPE, "type2": types.ANY_TYPE, "expected": False},
            {"type1": types.FLOAT_TYPE, "type2": types.INT_TYPE, "expected": False},
            {
                "type1": objpkg.KCLSchemaTypeObject(
                    name="Person",
                    runtime_type="runtime_type_543fa9efacae37b4c698a94214cdf779_Person",
                ),
                "type2": objpkg.KCLSchemaTypeObject(
                    name="Person",
                    runtime_type="runtime_type_543fa9efacae37b4c698a94214cdf779_Person",
                ),
                "expected": True,
            },
        ]
        for case in cases:
            type1, type2, expected = case["type1"], case["type2"], case["expected"]
            self.assertEqual(
                types.assignable_to(type1, type2),
                expected,
                msg=f"assert error on types {type1} and {type2}",
            )

    def test_type_to_kcl_type_annotation_str_invalid(self):
        with self.assertRaises(Exception):
            types.type_to_kcl_type_annotation_str(None)

        self.assertEqual(
            types.type_to_kcl_type_annotation_str(
                objpkg.KCLFunctionTypeObject("test", None, None, None)
            ),
            "",
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
