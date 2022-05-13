# Copyright 2021 The KCL Authors. All rights reserved.
import ast
import unittest
import pathlib

import kclvm.compiler.parser as parser
import kclvm.compiler.check.check_type as check_type
import kclvm.compiler.extension.builtin as builtin
import kclvm.api.object as objpkg
import kclvm.kcl.types as types
import kclvm.kcl.error as kcl_error
import kclvm.internal.util.check_utils as check_utils

path = pathlib.Path(__file__).parent
invalid_case_path_list = [
    "assert",
    "attr_op",
    "calculation",
    "final",
    "for_comp",
    "func_call",
    "if",
    "import",
    "loop",
    "module",
    "schema",
    "select_attr",
    "subscript",
    "type_alias",
    "type_annotation",
    "type_as",
    "unificaion",
    "unique",
    "unpack",
]
simple_case_file = str(path.joinpath("scope_test_data/simple.k"))
schema_case_file = str(path.joinpath("scope_test_data/schema.k"))
package_case_file = str(path.joinpath("scope_test_data/package.k"))
normal_cases = list(path.joinpath("normal_test_data").glob("*.k"))
err_collect_cases = list(path.joinpath("err_collect_test_data").glob("*.k"))
invalid_cases = [file for p in invalid_case_path_list for file in path.joinpath(f"invalid_test_data/{p}").glob("*.k")]


class TypeCheckerTest(unittest.TestCase):

    def test_internal_bug(self):
        check_utils.CHECK_MODE = True
        program = parser.LoadProgram(simple_case_file)
        tc = types.checker.TypeChecker(program)
        tc.config_expr_context.append("invalid_expr")
        with self.assertRaises(
            AssertionError,
            msg=f"Here is unreachable unless a bug occurs"
        ):
            tc.find_schema_attr_obj_from_schema_expr_stack("test")

    def test_switch_config_expr_context_by_key(self):
        check_utils.CHECK_MODE = True
        program = parser.LoadProgram(simple_case_file)
        tc = types.checker.TypeChecker(program)
        self.assertEqual(0, tc.switch_config_expr_context_by_key(ast.AST()))

    def test_clear_config_expr_context(self):
        check_utils.CHECK_MODE = True
        program = parser.LoadProgram(simple_case_file)
        tc = types.checker.TypeChecker(program)
        tc.config_expr_context.append("invalid_expr")
        self.assertEqual(1, len(tc.config_expr_context))
        tc.config_expr_context.append("invalid_expr")
        tc.config_expr_context.append("invalid_expr")
        self.assertEqual(3, len(tc.config_expr_context))
        tc.clear_config_expr_context(clear_all=True)
        self.assertEqual(0, len(tc.config_expr_context))

    def test_type_checker_simple_case(self):
        program = parser.LoadProgram(simple_case_file)
        prog_scope = types.ResolveProgram(program)
        scope = prog_scope.main_scope
        pkgpaths = prog_scope.pkgpaths
        base_schema_type = scope.elems["Base"].type.schema_type
        person_schema_type = scope.elems["Person"].type.schema_type
        self.assertListEqual(pkgpaths, ["__main__"])
        self.assertEqual(scope.elems["Base"].type.schema_type.type_str(), "Base")
        self.assertEqual(scope.elems["Person"].type.schema_type.type_str(), "Person")
        self.assertEqual(scope.parent, types.BUILTIN_SCOPE)
        self.assertEqual(len(scope.parent.elems), len(builtin.BUILTIN_FUNCTIONS))
        self.assertIsInstance(base_schema_type, objpkg.KCLSchemaTypeObject)
        self.assertIsInstance(person_schema_type, objpkg.KCLSchemaTypeObject)
        self.assertEqual(base_schema_type.name, "Base")
        self.assertEqual(person_schema_type.name, "Person")
        self.assertEqual(base_schema_type.base, None)
        self.assertIsInstance(person_schema_type.base, objpkg.KCLSchemaTypeObject)
        self.assertEqual(person_schema_type.base.name, "Base")

    def test_type_checker_schema_case(self):
        program = parser.LoadProgram(schema_case_file)
        scope = types.ResolveProgram(program).main_scope
        person_schema_type = scope.elems["Person"].type.schema_type
        self.assertIsInstance(person_schema_type, objpkg.KCLSchemaTypeObject)

    def test_type_checker_package_case(self):
        program = parser.LoadProgram(package_case_file)
        scope = types.ResolveProgram(program).main_scope
        person1_obj = scope.elems["person1"].type
        person2_obj = scope.elems["person2"].type
        self.assertIsInstance(person1_obj, objpkg.KCLSchemaTypeObject)
        self.assertIsInstance(person2_obj, objpkg.KCLSchemaTypeObject)
        self.assertEqual(package_case_file in scope.file_begin_position_map, True)
        self.assertEqual(scope.file_begin_position_map[package_case_file].filename, package_case_file)
        self.assertEqual(scope.file_begin_position_map[package_case_file].line, 1)
        self.assertEqual(scope.file_begin_position_map[package_case_file].column, 1)
        self.assertEqual(package_case_file in scope.file_end_position_map, True)
        self.assertEqual(scope.file_end_position_map[package_case_file].filename, package_case_file)
        self.assertEqual(scope.file_end_position_map[package_case_file].line, 12)
        self.assertEqual(scope.file_end_position_map[package_case_file].column, 2)
        self.assertEqual(person1_obj.pkgpath, "scope_test_data.pkg")
        self.assertEqual(person2_obj.pkgpath, "scope_test_data.pkg.pkg")

        for person_obj in [person1_obj, person2_obj]:
            self.assertTrue("__settings__" in person_obj.attr_obj_map)
            self.assertTrue("name" in person_obj.attr_obj_map)
            self.assertTrue("age" in person_obj.attr_obj_map)
            self.assertEqual(types.DICT_STR_ANY_TYPE, person_obj.attr_obj_map["__settings__"].attr_type)
            self.assertEqual(types.STR_TYPE, person_obj.attr_obj_map["name"].attr_type)
            self.assertEqual(types.INT_TYPE, person_obj.attr_obj_map["age"].attr_type)

    def test_type_checker_normal_case(self):
        for case in normal_cases:
            program = parser.LoadProgram(case)
            types.ResolveProgram(program)

    def test_type_checker_invalid_case(self):
        for case in invalid_cases:
            with self.assertRaises(
                kcl_error.KCLException, msg=f"case: {case}"
            ):
                program = parser.LoadProgram(case)
                types.ResolveProgram(program)

    def test_type_checker_err_collect_case(self):
        for case in err_collect_cases:
            program = parser.LoadProgram(case)
            types.ResolveProgram(program, config=types.CheckConfig(
                raise_err=False,
            ))

    def test_check_type(self):
        cases = [
            # True cases
            {"value": None, "expected_type": "int", "result": True},
            {"value": objpkg.Undefined, "expected_type": "int", "result": True},
            {"value": 1, "expected_type": "float", "result": True},
            {"value": 1, "expected_type": "", "result": True},
            {"value": 1, "expected_type": "int", "result": True},
            {"value": 1.1, "expected_type": "float", "result": True},
            {"value": "s", "expected_type": "str", "result": True},
            {"value": True, "expected_type": "bool", "result": True},
            {"value": [1, 2, 3], "expected_type": "[int]", "result": True},
            {"value": {"key": "value"}, "expected_type": "{str:}", "result": True},
            # False cases
            {"value": 1, "expected_type": "str", "result": False},
            {"value": 1.1, "expected_type": "int", "result": False},
            {"value": "s", "expected_type": "int", "result": False},
            {"value": True, "expected_type": "str", "result": False},
            {"value": [1, 2, 3], "expected_type": "[str]", "result": False},
            {"value": {"key": "value"}, "expected_type": "{str:int}", "result": False},
        ]
        for case in cases:
            value = objpkg.to_kcl_obj(case["value"])
            expected_type = case["expected_type"]
            result = case["result"]
            self.assertEqual(
                check_type.check_type(value, expected_type)[0],
                result,
                msg=f"value: {value}, expected_type: {expected_type}"
            )

    def test_check_type_builtin(self):
        cases = [
            # True cases
            {"value": 1, "expected_types": [], "result": True},
            {"value": 1, "expected_types": ["float"], "result": True},
            {"value": 1, "expected_types": ["int"], "result": True},
            {"value": 1, "expected_types": ["str", "int"], "result": True},
            {"value": 1.1, "expected_types": ["float"], "result": True},
            {"value": 1.1, "expected_types": ["float", "int"], "result": True},
            {"value": "s", "expected_types": ["str"], "result": True},
            {"value": True, "expected_types": ["bool"], "result": True},
            # False cases
            {"value": 1, "expected_types": ["str"], "result": False},
            {"value": 1.1, "expected_types": ["int"], "result": False},
            {"value": "s", "expected_types": ["int"], "result": False},
            {"value": True, "expected_types": ["str"], "result": False},
        ]
        for case in cases:
            value = objpkg.to_kcl_obj(case["value"])
            expected_types = case["expected_types"]
            result = case["result"]
            self.assertEqual(
                check_type.check_type_builtin(value, expected_types, should_raise_err=False),
                result,
                msg=f"value: {value}, expected_types: {expected_types}"
            )
            

if __name__ == "__main__":
    unittest.main(verbosity=2)
