#! /usr/bin/env python3

import unittest

from kclvm.compiler.astutil import filter_declarations
from kclvm.compiler.parser import ParseFile

import kclvm.kcl.ast as ast
import kclvm.kcl.error as kcl_error

code = """
schema Person:
    name: str
    age: int

schema Config:
    data?: [int]

a = 1
b = 2
person = Person {
    name: "Alice"
    age: 18
}
config = Config {}
"""


class KCLASTFilterTest(unittest.TestCase):
    """
    KCL AST filter test
    """

    def test_filter(self):
        module = ParseFile("test.k", code)
        global_declarations = filter_declarations(module)
        schema_declarations = filter_declarations(module, ast.SchemaExpr)
        binary_declarations = filter_declarations(module, ast.BinaryExpr)
        self.assertEqual(len(global_declarations), 4)
        self.assertEqual(len(schema_declarations), 2)
        self.assertEqual(len(binary_declarations), 0)
        self.assertEqual(global_declarations[0].name, "a")
        self.assertEqual(global_declarations[1].name, "b")
        self.assertEqual(global_declarations[2].name, "person")
        self.assertEqual(global_declarations[3].name, "config")
        self.assertEqual(schema_declarations[0].name, "person")
        self.assertEqual(schema_declarations[1].name, "config")


if __name__ == "__main__":
    unittest.main(verbosity=2)
