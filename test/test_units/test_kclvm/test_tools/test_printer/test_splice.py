#! /usr/bin/env python3

import os
import io
import unittest
import pathlib
from typing import Tuple

from kclvm.tools.printer import SchemaRuleCodeSnippet, splice_schema_with_rule
from kclvm.tools.printer.splice import (
    build_rule_check_block_str,
    add_indent_to_code_string,
)


class TestSplice(unittest.TestCase):
    def test_splice_schema_with_rule(self):
        cases = [
            {
                "snippet_list": [
                    SchemaRuleCodeSnippet(
                        schema="""\
schema Person:
    name: str
""",
                        rule="""\
"a" in name
""",
                    )
                ],
                "expected": """\
schema Person:
    name: str

    check:
        "a" in name
""",
            },
            {
                "snippet_list": [
                    SchemaRuleCodeSnippet(
                        schema="""\
# Schema Person definition
schema Person:
    name: str
""",
                        rule="""\
# Schema Person Rule definition
"a" in name
""",
                    )
                ],
                "expected": """\
# Schema Person definition
schema Person:
    name: str

    check:
        # Schema Person Rule definition
        "a" in name
""",
            },
            {
                "snippet_list": [
                    SchemaRuleCodeSnippet(
                        schema="""\
schema Person:
    name: str
""",
                        rule="""""",
                    )
                ],
                "expected": """\
schema Person:
    name: str
""",
            },
            {
                "snippet_list": [
                    SchemaRuleCodeSnippet(
                        schema="""""",
                        rule="""\
"a" in name
""",
                    )
                ],
                "expected": """
""",
            },
            {
                "snippet_list": [
                    SchemaRuleCodeSnippet(
                        schema="""\
schema Person:
    name: str
    data: Data
""",
                        rule="""\
"a" in name
""",
                    ),
                    SchemaRuleCodeSnippet(
                        schema="""\
schema Data:
    id: int
""",
                        rule="""\
id > 0
""",
                    ),
                ],
                "expected": """\
schema Person:
    name: str
    data: Data

    check:
        "a" in name

schema Data:
    id: int

    check:
        id > 0
""",
            },
        ]
        for case in cases:
            snippet_list, expected = case["snippet_list"], case["expected"]
            self.assertEqual(
                splice_schema_with_rule(snippet_list), expected, msg=f"{snippet_list}"
            )

    def test_splice_schema_with_rule_value_error(self):
        cases = [
            {"snippet_list": None},
            {"snippet_list": 1},
            {"snippet_list": [1]},
        ]
        for case in cases:
            snippet_list = case["snippet_list"]
            with self.assertRaises(ValueError):
                splice_schema_with_rule(snippet_list)

    def test_build_rule_check_block_str(self):
        cases = [
            {"schema": "Mock", "code": "", "expected": ""},
            {"schema": "", "code": "a > 1", "expected": ""},
            {
                "schema": "Mock",
                "code": "a > 1",
                "expected": """\
schema Mock:
    check:
        a > 1\
""",
            },
            {
                "schema": "Mock",
                "code": "a > 1\nb < 1",
                "expected": """\
schema Mock:
    check:
        a > 1
        b < 1\
""",
            },
        ]
        for case in cases:
            schema, code, expected = case["schema"], case["code"], case["expected"]
            self.assertEqual(
                build_rule_check_block_str(schema, code),
                expected,
                msg=f"schema: {schema}, code: {code}",
            )

    def test_add_indent_to_code_string(self):
        cases = [
            {"code": None, "indent": 2, "expected": ""},
            {"code": "", "indent": 2, "expected": ""},
            {"code": "a = 1", "indent": 2, "expected": "  a = 1"},
            {"code": "a = 1", "indent": 4, "expected": "    a = 1"},
            {"code": "a = 1", "indent": 8, "expected": "        a = 1"},
            {"code": "a = 1\nb = 1", "indent": 4, "expected": "    a = 1\n    b = 1"},
        ]
        for case in cases:
            code, indent, expected = case["code"], case["indent"], case["expected"]
            self.assertEqual(
                add_indent_to_code_string(code, indent), expected, msg=f"{code}"
            )


if __name__ == "__main__":
    unittest.main(verbosity=2)
