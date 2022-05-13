# Copyright 2020 The KCL Authors. All rights reserved.

import os
import pathlib
import unittest

import kclvm.api.object.internal as internal
import kclvm.kcl.ast as ast
import kclvm.compiler.astutil.builder as ast_builder


class TestASTUtil(unittest.TestCase):
    def get_fake_schema_expr(self, name: str) -> ast.SchemaExpr:
        schema_expr = ast.SchemaExpr(line=1, column=1)
        schema_name = ast.Identifier(line=1, column=1)
        schema_name.names = [name]
        schema_expr.name = schema_name
        schema_expr.config = ast.ConfigExpr(line=1, column=1 + 1 +len(name))
        return schema_expr

    def test_build_lit_node_from_string(self):
        cases = [
            {"value": "a", "expected": ast.StringLit(value="a", line=1, column=1)},
            {"value": "'a'", "expected": ast.StringLit(value="a", line=1, column=1)},
            {"value": '"a"', "expected": ast.StringLit(value="a", line=1, column=1)},
            {"value": "kclvm:v1", "expected": ast.StringLit(value="kclvm:v1", line=1, column=1)},
            {"value": "1", "expected": ast.NumberLit(value=1, line=1, column=1)},
            {"value": "1.1", "expected": ast.NumberLit(value=1.1, line=1, column=1)},
            {"value": "1.0e1", "expected": ast.NumberLit(value=10.0, line=1, column=1)},
            {"value": "True", "expected": ast.NameConstantLit(value=True, line=1, column=1)},
            {"value": "False", "expected": ast.NameConstantLit(value=False, line=1, column=1)},
            {"value": "None", "expected": ast.NameConstantLit(value=None, line=1, column=1)},
            {"value": "Undefined", "expected": ast.NameConstantLit(value=internal.Undefined, line=1, column=1)},
            {"value": "[]", "expected": ast.ListExpr(line=1, column=1)},
            {"value": "{}", "expected": ast.ConfigExpr(line=1, column=1)},
            {"value": "Data {}", "expected": self.get_fake_schema_expr("Data")},
            {"value": "pkg.Data {}", "expected": self.get_fake_schema_expr("pkg.Data")},
        ]
        for case in cases:
            value, expected = case["value"], case["expected"]
            ast_node = ast_builder.BuildNodeFromString(value, 1, 1)
            self.assertEqual(str(ast_node), str(expected))

    def test_build_lit_node_from_value(self):
        cases = [
            {"value": "a", "expected": ast.StringLit(value="a", line=1, column=1)},
            {"value": "'a'", "expected": ast.StringLit(value="'a'", line=1, column=1)},
            {"value": '"a"', "expected": ast.StringLit(value="\"a\"", line=1, column=1)},
            {"value": "kclvm:v1", "expected": ast.StringLit(value="kclvm:v1", line=1, column=1)},
            {"value": 1, "expected": ast.NumberLit(value=1, line=1, column=1)},
            {"value": 1.1, "expected": ast.NumberLit(value=1.1, line=1, column=1)},
            {"value": 1.0e1, "expected": ast.NumberLit(value=10.0, line=1, column=1)},
            {"value": True, "expected": ast.NameConstantLit(value=True, line=1, column=1)},
            {"value": False, "expected": ast.NameConstantLit(value=False, line=1, column=1)},
            {"value": None, "expected": ast.NameConstantLit(value=None, line=1, column=1)},
            {"value": internal.Undefined, "expected": ast.NameConstantLit(value=internal.Undefined, line=1, column=1)},
        ]
        for case in cases:
            value, expected = case["value"], case["expected"]
            ast_node = ast_builder.BuildLitNodeFromValue(value, 1, 1)
            self.assertEqual(str(ast_node), str(expected))


if __name__ == "__main__":
    unittest.main(verbosity=2)
