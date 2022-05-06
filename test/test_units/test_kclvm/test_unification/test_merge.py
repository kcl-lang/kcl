#! /usr/bin/env python3

import unittest

import kclvm.kcl.ast as ast
import kclvm.kcl.error as kcl_error
from kclvm.compiler.parser import ParseFile
from kclvm.unification import MergeAST, MergeASTList

codes = [
    """
schema Person:
    name: str
    age: int
    labels: {str:}

person = Person {
    name: "Alice"
}
person = Person {
    labels.key: "value"
}
""",
    """
person = Person {
    age: 18
}
persons = Person.instances()
""",
]
type_merge_not_conflict_case = """\
schema Id1:
    id1?: int

schema Id2:
    id2?: int

schema Data:
    id?: Id1 | Id2

data = Data {
    id = Id1 {id1 = 1}
    id = Id2 {id2 = 2}
}
"""
type_merge_conflict_case = """\
schema Id1:
    id1?: int

schema Id2:
    id2?: int

schema Data:
    id?: Id1 | Id2

data = Data {
    id: Id1 {id1 = 1}
    id: Id2 {id2 = 2}
}
"""


class KCLMergeTest(unittest.TestCase):
    def setUp(self):
        self.maxDiff = None
        self.modules = [ParseFile(f"test-{i}.k", code) for i, code in enumerate(codes)]
        return super().setUp()

    def test_merge(self):
        module = MergeAST(self.modules[0])
        self.assertEqual(len(module.body), 2)
        self.assertIsInstance(module.body[0], ast.SchemaStmt)
        self.assertIsInstance(module.body[1], ast.AssignStmt)
        self.assertEqual(len(module.body[1].value.config.items), 2)
        self.assertEqual(module.body[1].value.config.keys[0].get_name(), "name")
        self.assertEqual(module.body[1].value.config.keys[1].get_name(), "labels")

    def test_merge_list(self):
        modules = MergeASTList(self.modules)
        self.assertEqual(len(modules), 2)
        # The first module
        self.assertEqual(len(modules[0].body), 1)
        self.assertIsInstance(modules[0].body[0], ast.SchemaStmt)
        # The second module
        self.assertIsInstance(modules[1].body[0], ast.AssignStmt)
        self.assertEqual(len(modules[1].body[0].value.config.items), 3)
        self.assertEqual(modules[1].body[0].value.config.keys[0].get_name(), "name")
        self.assertEqual(modules[1].body[0].value.config.keys[1].get_name(), "labels")
        self.assertEqual(modules[1].body[0].value.config.keys[2].get_name(), "age")

    def test_merge_type_conflict(self):
        module = ParseFile("type_merge_not_conflict_case", type_merge_not_conflict_case)
        module = MergeASTList([module])
        module = ParseFile("type_merge_conflict_case", type_merge_conflict_case)
        with self.assertRaises(kcl_error.CompileError) as err:
            module = MergeASTList([module])
        self.assertIn("conflict unification types between Id2 and Id1", str(err.exception))


if __name__ == "__main__":
    unittest.main(verbosity=2)
