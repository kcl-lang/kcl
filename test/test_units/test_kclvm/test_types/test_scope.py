# Copyright 2021 The KCL Authors. All rights reserved.

import unittest
import pathlib
import typing

import kclvm.kcl.ast as ast
import kclvm.compiler.parser as parser
import kclvm.kcl.types as types


path = pathlib.Path(__file__).parent
simple_case_file = str(path.joinpath("scope_test_data/simple.k"))


class ScopeTest(unittest.TestCase):
    def test_scope_inner_most(self):
        program = parser.LoadProgram(simple_case_file)
        scope = types.ResolveProgram(program).main_scope
        while scope.parent is not None:
            scope = scope.parent
        inner_most = scope.inner_most(pos=ast.Position(filename=simple_case_file, line=6, column=2))
        self.assertIsNotNone(inner_most)
        self.assertTrue(isinstance(inner_most.node, ast.SchemaStmt))
        self.assertTrue(typing.cast(ast.SchemaStmt, inner_most.node).name == "Person")

    def test_scope_contains_pos(self):
        program = parser.LoadProgram(simple_case_file)
        scope = types.ResolveProgram(program).main_scope
        self.assertTrue(isinstance(scope, types.PackageScope))
        # Schema Statement
        self.assertTrue(scope.contains_pos(pos=ast.Position(filename=simple_case_file, line=3, column=2)))
        # Rule Statement
        self.assertTrue(scope.contains_pos(pos=ast.Position(filename=simple_case_file, line=30, column=5)))
        for child in scope.children:
            if isinstance(child.node, ast.SchemaStmt) and child.node.name == "Base":
                self.assertTrue(child.contains_pos(pos=ast.Position(filename=simple_case_file, line=3, column=2)))
            elif isinstance(child.node, ast.RuleStmt) and child.node.name == "SomeRule":
                self.assertTrue(child.contains_pos(pos=ast.Position(filename=simple_case_file, line=30, column=5)))
                self.assertFalse(child.contains_pos(pos=ast.Position(filename=simple_case_file, line=28, column=5)))

if __name__ == "__main__":
    unittest.main(verbosity=2)
