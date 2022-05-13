# Copyright 2021 The KCL Authors. All rights reserved.

import unittest
from typing import cast

import kclvm.kcl.ast as ast
from kclvm.compiler.parser import ParseFile
from kclvm.compiler.build.preprocess import fix_identifier_prefix


simple_case = """
schema $all:
    data?: int

schema $filter($all):
    $name: str

$map = $filter {
    name: "Data"
}
"""

if_case = """
schema Data:
    if False:
        $name = "1"
    elif True:
        $name = "2"
    else:
        $name = "3"
"""

import_case = """
import $import.$all as $all
"""


class CompilerPreprocessTest(unittest.TestCase):
    def test_fix_identifier_prefix_invalid_case(self):
        self.assertEqual(fix_identifier_prefix(None), None)

    def test_fix_identifier_prefix_simple_case(self):
        module = ParseFile("", code=simple_case)
        module = cast(ast.Module, fix_identifier_prefix(module))
        self.assertEqual(len(module.body), 3)
        schema_stmt_all: ast.SchemaStmt = cast(ast.SchemaStmt, module.body[0])
        schema_stmt_filter: ast.SchemaStmt = cast(ast.SchemaStmt, module.body[1])
        assign_stmt: ast.AssignStmt = cast(ast.AssignStmt, module.body[2])
        self.assertEqual(schema_stmt_all.name, "all")
        self.assertEqual(schema_stmt_filter.name, "filter")
        self.assertEqual(schema_stmt_filter.parent_name.get_name(), "all")
        self.assertEqual(schema_stmt_filter.body[0].name, "name")
        self.assertEqual(assign_stmt.value.name.get_name(), "filter")

    def test_fix_identifier_prefix_if_case(self):
        module = ParseFile("", code=if_case)
        module = cast(ast.Module, fix_identifier_prefix(module))
        self.assertEqual(len(module.body), 1)
        schema_stmt: ast.SchemaStmt = cast(ast.SchemaStmt, module.body[0])
        self.assertEqual(len(schema_stmt.body), 1)
        if_stmt: ast.IfStmt = cast(ast.IfStmt, schema_stmt.body[0])
        self.assertEqual(if_stmt.body[0].targets[0].get_name(), "name")
        self.assertEqual(if_stmt.body[0].value.value, "1")
        self.assertEqual(if_stmt.elif_body[0][0].targets[0].get_name(), "name")
        self.assertEqual(if_stmt.elif_body[0][0].value.value, "2")
        self.assertEqual(if_stmt.else_body[0].targets[0].get_name(), "name")
        self.assertEqual(if_stmt.else_body[0].value.value, "3")

    def test_fix_identifier_prefix_import_case(self):
        module = ParseFile("", code=import_case)
        module = cast(ast.Module, fix_identifier_prefix(module))
        self.assertEqual(len(module.body), 1)
        import_stmt: ast.ImportStmt = cast(ast.ImportStmt, module.body[0])
        self.assertEqual(import_stmt.asname, "all")
        self.assertEqual(import_stmt.name, "all")
        self.assertEqual(import_stmt.pkg_name, "all")
        self.assertEqual(import_stmt.path, "import.all")
        self.assertEqual(len(import_stmt.path_nodes), 2)
        self.assertEqual(import_stmt.path_nodes[0].value, "import")
        self.assertEqual(import_stmt.path_nodes[1].value, "all")



if __name__ == "__main__":
    unittest.main(verbosity=2)
