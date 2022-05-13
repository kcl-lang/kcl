# Copyright 2020 The KCL Authors. All rights reserved.

import os
import pathlib
import unittest
import typing

import kclvm.kcl.ast as ast
import kclvm.compiler.parser as parser
import kclvm.compiler.astutil.fix as fix

MAIN_FILE = "main.k"
MAIN_PKG = "__main__"
ROOT_PATH = str(pathlib.Path(__file__).parent)

fix_test_schema_relaxed_case = """
import path.to.pkg as pkgname 

x = pkgname.Name

schema Person:
    name: str
    age: int

schema TestPerson:
    name = "Alice"
    age = 18
    person: Person = Person {
        name: name
        age: age
    }
    assert person.name == name
    assert person.age == age
"""
fix_module_import_list_case = """
import path.to.pkg as pkgname
import another.path.to.pkg as another_pkgname

x = pkgname.Name

schema Person[dataVar: pkgname.Data | another_pkgname.Version]:
    name: str
    age: int
    data?: pkgname.Data | another_pkgname.Version = dataVar

rule PersonCheck[data: pkgname.Data]:
    data.id > 0

func = lambda x: pkgname.Data, y -> pkgname.Data {
    x + y
}

person = Person(pkgname.Data {id = 1}) {
    name = "Alice"
    age = 18
}
var: pkgname.Data = pkgname.Data {}
type Data = pkgname.Data
"""


class TestFixQualifiedIdentifier(unittest.TestCase):
    def test_fix_qualified_identifier(self):
        module = parser.ParseFile(
            MAIN_FILE, code=fix_test_schema_relaxed_case, pkg=MAIN_PKG
        )
        fix.fix_qualified_identifier(module)


class TestFixAndGetModuleImportList(unittest.TestCase):
    def test_fix_and_get_module_import_list(self):
        module = parser.ParseFile(
            MAIN_FILE, code=fix_module_import_list_case, pkg=MAIN_PKG
        )
        self.assertIsInstance(module.body[0], ast.ImportStmt)
        self.assertIsInstance(module.body[1], ast.ImportStmt)
        self.assertIsInstance(module.body[2], ast.AssignStmt)
        self.assertIsInstance(module.body[3], ast.SchemaStmt)
        self.assertIsInstance(module.body[4], ast.RuleStmt)
        self.assertIsInstance(module.body[5].value, ast.LambdaExpr)
        self.assertIsInstance(module.body[6], ast.AssignStmt)
        self.assertIsInstance(module.body[7], ast.AssignStmt)
        self.assertIsInstance(module.body[8], ast.TypeAliasStmt)
        schema_args = typing.cast(ast.SchemaStmt, module.body[3]).args
        self.assertEqual(schema_args.GetArgType(0), "pkgname.Data|another_pkgname.Version")
        import_list = fix.fix_and_get_module_import_list(ROOT_PATH, module)
        self.assertEqual(len(import_list), 2)
        self.assertEqual(import_list[0].path, "path.to.pkg")
        self.assertEqual(import_list[0].asname, "pkgname")
        self.assertEqual(schema_args.GetArgType(0), "@path.to.pkg.Data|@another.path.to.pkg.Version")
        fix.fix_and_get_module_import_list(ROOT_PATH, module, reversed=True)
        self.assertEqual(schema_args.GetArgType(0), "pkgname.Data|another_pkgname.Version")


class TestFixSchemaAutoRelaxed(unittest.TestCase):
    def test_fix_test_schema_auto_relaxed_invalid(self):
        module = parser.ParseFile("invalid_name.k", code=fix_test_schema_relaxed_case)
        # Before fix
        self.assertEqual(len(module.body), 4)
        self.assertIsInstance(module.body[3], ast.SchemaStmt)
        # After fix
        fix.fix_test_schema_auto_relaxed(module)
        self.assertEqual(len(module.body), 4)
        self.assertIsInstance(module.body[3], ast.SchemaStmt)

    def test_fix_test_schema_auto_relaxed_normal(self):
        module = parser.ParseFile("person_test.k", code=fix_test_schema_relaxed_case)
        # Before fix
        self.assertEqual(len(module.body), 4)
        self.assertIsInstance(module.body[3], ast.SchemaStmt)
        # After fix
        fix.fix_test_schema_auto_relaxed(module)
        self.assertEqual(len(module.body), 4)
        self.assertIsInstance(module.body[3], ast.SchemaStmt)


if __name__ == "__main__":
    unittest.main(verbosity=2)
