# Copyright 2020 The KCL Authors. All rights reserved.

import os
import pathlib
import unittest

import kclvm.kcl.ast as ast
import kclvm.compiler.parser as parser
import kclvm.compiler.astutil.filter as filter

filter_simple_case = """
schema Config:
    id: int

config1 = Config {id = 1}
config2: Config {id = 2}
config3 = {id = 3}
"""


class TestASTFilter(unittest.TestCase):
    def test_filter_declarations(self):
        module = parser.ParseFile(
            "__main__.k", code=filter_simple_case
        )
        declarations = filter.filter_declarations(module)
        self.assertEqual(len(declarations), 3)
        declarations = filter.filter_declarations(module, ast_type=ast.SchemaExpr)
        self.assertEqual(len(declarations), 2)
        declarations = filter.filter_declarations(module, ast_type="SchemaExpr")
        self.assertEqual(len(declarations), 2)
        declarations = filter.filter_declarations(module, ast_type=(ast.SchemaExpr, ast.ConfigExpr))
        self.assertEqual(len(declarations), 3)


if __name__ == "__main__":
    unittest.main(verbosity=2)
