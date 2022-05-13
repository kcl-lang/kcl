# Copyright 2021 The KCL Authors. All rights reserved.

import unittest
import pathlib
from typing import Tuple

import kclvm.api.object as objpkg
import kclvm.api.object.internal.common as common
import kclvm.kcl.types as types
import kclvm.kcl.types.type_parser as type_parser


class TypeParserTest(unittest.TestCase):
    def test_is_lit_type(self):
        cases = [
            {"type_str": "1", "expected": True},
            {"type_str": "1.1", "expected": True},
            {"type_str": "1.e+0", "expected": True},
            {"type_str": ".1", "expected": True},
            {"type_str": "True", "expected": True},
            {"type_str": "False", "expected": True},
            {"type_str": "'s'", "expected": True},
            {"type_str": "\"s\"", "expected": True},
            {"type_str": "true", "expected": False},
            {"type_str": "false", "expected": False},
            {"type_str": "\"s", "expected": False},
            {"type_str": "'s", "expected": False},
            {"type_str": "schema", "expected": False},
            {"type_str": "pkg.schema", "expected": False},
        ]
        for case in cases:
            type_str, expected = case["type_str"], case["expected"]
            self.assertEqual(
                type_parser.is_lit_type_str(type_str), expected
            )

    def test_is_type_union(self):
        cases = [
            {"type_str": "A|B|C", "expected": True},
            {"type_str": "'123'|'456'|'789'", "expected": True},
            {"type_str": "'|'|'||'|'|||'", "expected": True},
            {"type_str": '"aa\\"ab|"|"aa\\"abccc"', "expected": True},
            {"type_str": '["|"]|""', "expected": True},
            {"type_str": '{str:"|"}|"|"', "expected": True},
            {"type_str": '"aa\\"ab|"', "expected": False},
            {"type_str": '"|aa\\"ab|"', "expected": False},
        ]
        for case in cases:
            type_str, expected = case["type_str"], case["expected"]
            self.assertEqual(
                common.is_type_union(type_str), expected
            )

    def test_split_type_union(self):
        cases = [
            {"type_str": "A|B|C", "expected": ["A", "B", "C"]},
            {"type_str": "'123'|'456'|'789'", "expected": ["'123'", "'456'", "'789'"]},
            {"type_str": "'|'|'||'|'|||'", "expected": ["'|'", "'||'", "'|||'"]},
            {"type_str": '["|"]|""', "expected": ['["|"]', '""']},
            {"type_str": '{str:"|"}|"|"', "expected": ['{str:"|"}', '"|"']},
        ]
        for case in cases:
            type_str, expected = case["type_str"], case["expected"]
            self.assertEqual(
                common.split_type_union(type_str), expected
            )

    def test_parse_type_str_normal(self):
        cases = [
            # Common built-in types
            {"type_str": None, "expected": types.ANY_TYPE},
            {"type_str": "", "expected": types.ANY_TYPE},
            {"type_str": "any", "expected": types.ANY_TYPE},
            {"type_str": "any", "expected": objpkg.KCLAnyTypeObject()},
            {"type_str": "int", "expected": types.INT_TYPE},
            {"type_str": "float", "expected": types.FLOAT_TYPE},
            {"type_str": "str", "expected": types.STR_TYPE},
            {"type_str": "bool", "expected": types.BOOL_TYPE},
            # Dict types
            {
                "type_str": "{:}",
                "expected": objpkg.KCLDictTypeObject(
                    key_type=types.ANY_TYPE, value_type=types.ANY_TYPE
                ),
            },
            {
                "type_str": "{str:}",
                "expected": objpkg.KCLDictTypeObject(
                    key_type=types.STR_TYPE, value_type=types.ANY_TYPE
                ),
            },
            {
                "type_str": "{str:any}",
                "expected": objpkg.KCLDictTypeObject(
                    key_type=types.STR_TYPE, value_type=types.ANY_TYPE
                ),
            },
            {
                "type_str": "{str:str}",
                "expected": objpkg.KCLDictTypeObject(
                    key_type=types.STR_TYPE, value_type=types.STR_TYPE
                ),
            },
            {
                "type_str": "{str:{str:str}}",
                "expected": objpkg.KCLDictTypeObject(
                    key_type=types.STR_TYPE,
                    value_type=objpkg.KCLDictTypeObject(
                        key_type=types.STR_TYPE,
                        value_type=types.STR_TYPE,
                    ),
                ),
            },
            {
                "type_str": "{str:[str]}",
                "expected": objpkg.KCLDictTypeObject(
                    key_type=types.STR_TYPE,
                    value_type=objpkg.KCLListTypeObject(item_type=types.STR_TYPE),
                ),
            },
            # List types
            {
                "type_str": "[]",
                "expected": objpkg.KCLListTypeObject(item_type=types.ANY_TYPE),
            },
            {
                "type_str": "[any]",
                "expected": objpkg.KCLListTypeObject(item_type=types.ANY_TYPE),
            },
            {
                "type_str": "[str]",
                "expected": objpkg.KCLListTypeObject(item_type=types.STR_TYPE),
            },
            {
                "type_str": "[{str:}]",
                "expected": objpkg.KCLListTypeObject(item_type=types.DICT_STR_ANY_TYPE),
            },
            {
                "type_str": "[{str:str}]",
                "expected": objpkg.KCLListTypeObject(item_type=types.DICT_STR_STR_TYPE),
            },
            # Union types
            {
                "type_str": "str|int",
                "expected": objpkg.KCLUnionTypeObject(
                    types=[
                        types.INT_TYPE,
                        types.STR_TYPE,
                    ]
                ),
            },
            {
                "type_str": "int|str",
                "expected": objpkg.KCLUnionTypeObject(
                    types=[
                        types.INT_TYPE,
                        types.STR_TYPE,
                    ]
                ),
            },
            {
                "type_str": "int|str|int",
                "expected": objpkg.KCLUnionTypeObject(
                    types=[
                        types.INT_TYPE,
                        types.STR_TYPE,
                    ]
                ),
            },
            {
                "type_str": "int|str|int|str",
                "expected": objpkg.KCLUnionTypeObject(
                    types=[
                        types.INT_TYPE,
                        types.STR_TYPE,
                    ]
                ),
            },
            {
                "type_str": "{str:int|str}",
                "expected": objpkg.KCLDictTypeObject(
                    key_type=types.STR_TYPE,
                    value_type=objpkg.KCLUnionTypeObject(
                        types=[
                            types.INT_TYPE,
                            types.STR_TYPE,
                        ]
                    ),
                ),
            },
            {
                "type_str": "{str|int:int|str}",
                "expected": objpkg.KCLDictTypeObject(
                    key_type=objpkg.KCLUnionTypeObject(
                        types=[
                            types.INT_TYPE,
                            types.STR_TYPE,
                        ]
                    ),
                    value_type=objpkg.KCLUnionTypeObject(
                        types=[
                            types.INT_TYPE,
                            types.STR_TYPE,
                        ]
                    ),
                ),
            },
            {
                "type_str": "[int|str]",
                "expected": objpkg.KCLListTypeObject(
                    item_type=objpkg.KCLUnionTypeObject(
                        types=[
                            types.INT_TYPE,
                            types.STR_TYPE,
                        ]
                    ),
                ),
            },
            # Literal types
            {"type_str": "True", "expected": types.TRUE_LIT_TYPE},
            {"type_str": "False", "expected": types.FALSE_LIT_TYPE},
            {
                "type_str": "123",
                "expected": objpkg.KCLIntLitTypeObject(value=123),
            },
            {
                "type_str": "123.0",
                "expected": objpkg.KCLFloatLitTypeObject(value=123.0),
            },
            {
                "type_str": "'ss'",
                "expected": objpkg.KCLStringLitTypeObject(value="ss"),
            },
            {
                "type_str": '"ss"',
                "expected": objpkg.KCLStringLitTypeObject(value="ss"),
            },
            {
                "type_str": "'Red'|'Yellow'|'Blue'",
                "expected": objpkg.KCLUnionTypeObject(
                    types=[
                        objpkg.KCLStringLitTypeObject(value="Red"),
                        objpkg.KCLStringLitTypeObject(value="Yellow"),
                        objpkg.KCLStringLitTypeObject(value="Blue"),
                    ]
                ),
            },
            {
                "type_str": "1|2|3",
                "expected": objpkg.KCLUnionTypeObject(
                    types=[
                        objpkg.KCLIntLitTypeObject(value=1),
                        objpkg.KCLIntLitTypeObject(value=2),
                        objpkg.KCLIntLitTypeObject(value=3),
                    ]
                ),
            },
            # Partially ordered types
            {
                "type_str": "str|'ss'|int",
                "expected": objpkg.KCLUnionTypeObject(
                    types=[
                        types.INT_TYPE,
                        types.STR_TYPE,
                    ]
                ),
            },
            {
                "type_str": "str|'ss'|int|1|bool|True",
                "expected": objpkg.KCLUnionTypeObject(
                    types=[
                        types.BOOL_TYPE,
                        types.INT_TYPE,
                        types.STR_TYPE,
                    ]
                ),
            },
            {
                "type_str": "1|1|2",
                "expected": objpkg.KCLUnionTypeObject(
                    types=[
                        objpkg.KCLIntLitTypeObject(value=1),
                        objpkg.KCLIntLitTypeObject(value=2),
                    ]
                ),
            },
            {
                "type_str": "2|1|1",
                "expected": objpkg.KCLUnionTypeObject(
                    types=[
                        objpkg.KCLIntLitTypeObject(value=2),
                        objpkg.KCLIntLitTypeObject(value=1),
                    ]
                ),
            },
            {
                "type_str": "{str:}|{str:str}",
                "expected": types.DICT_STR_ANY_TYPE,
            },
            {
                "type_str": "[]|[str]",
                "expected": objpkg.KCLListTypeObject(item_type=types.ANY_TYPE),
            },
            {
                "type_str": '1|"aaa"|True',
                "expected": objpkg.KCLUnionTypeObject(
                    types=[
                        types.TRUE_LIT_TYPE,
                        objpkg.KCLIntLitTypeObject(1),
                        objpkg.KCLStringLitTypeObject("aaa"),
                    ]
                ),
            },
        ]
        for case in cases:
            type_str, expected = case["type_str"], case["expected"]
            self.assertEqual(
                types.parse_type_str(type_str),
                expected,
                msg=f"Assert error type: {type_str}",
            )

    def test_parse_type_str_invalid(self):
        cases = [
            {"type_str": True},
            {"type_str": 1},
            {"type_str": []},
            {"type_str": {}},
            {"type_str": ()},
        ]
        for case in cases:
            type_str = case["type_str"]
            with self.assertRaises(ValueError):
                types.parse_type_str(type_str)


if __name__ == "__main__":
    unittest.main(verbosity=2)
