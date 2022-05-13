#! /usr/bin/env python3

import os
import unittest
import pathlib
from typing import Tuple

import kclvm.kcl.ast as ast
from kclvm.compiler.parser import ParseFile
from kclvm.compiler.astutil.filter import filter_stmt
from kclvm.unification import MergeASTToVertex, MergeStrategy, Vertex


_FILE_INPUT_SUFFIX = ".input"
_FILE_VERTEX_SUFFIX = ".vertex"
_FILE_CODE_SUFFIX = ".code"
_PATH_NAME = "test_data"
_DIR_PATH = pathlib.Path(__file__).parent.joinpath(_PATH_NAME)
_TEST_CASE_NAMES = [
    "collection_if",
    "empty",
    "insert",
    "int_dict",
    "nest_declaration",
    "nest_var_0",
    "nest_var_1",
    "override",
    "schema_and_dict",
    "schema_with_list",
    "schema",
    "single",
    "str_interpolation",
    "unification",
    "unpack",
]


class KCLUnifierBaseTest(unittest.TestCase):
    def setUp(self):
        self.maxDiff = None
        self.test_cases = [self.read_data(case) for case in _TEST_CASE_NAMES]

    def read_data_with_suffix(self, data_name: str, suffix: str):
        return (_DIR_PATH / (data_name + suffix)).read_text()

    def read_data(self, data_name: str) -> Tuple[str, str]:
        """Read test data"""
        data_input = self.read_data_with_suffix(data_name, _FILE_INPUT_SUFFIX)
        data_vertex = self.read_data_with_suffix(data_name, _FILE_VERTEX_SUFFIX)
        data_code = self.read_data_with_suffix(data_name, _FILE_CODE_SUFFIX)
        return data_input, data_vertex, data_code

    def assert_vertex_unify_equal(
        self,
        input_str: str,
        vertex_str: str,
        code_str: str,
        strategy: MergeStrategy = MergeStrategy.UNION,
        msg=None,
    ):
        origin_module = ParseFile("", input_str)
        code_module = ParseFile("", code_str)
        origin_vertex, unify_vertex, merge_module = MergeASTToVertex(origin_module)
        # TODO: Optimize the function test to smaller granularity test. @lingzhi.xpf
        self.assertEqual(unify_vertex.pretty(), vertex_str, msg=msg)


class KCLUnifierTest(KCLUnifierBaseTest):
    def test_vertex_unification(self) -> None:
        for input, vertex, code in self.test_cases:
            self.assert_vertex_unify_equal(input, vertex, code, msg=f"the fail code is\n{code}")

    def test_vertex_override_merge(self) -> None:
        for input, vertex, code in self.test_cases:
            self.assert_vertex_unify_equal(input, vertex, code, MergeStrategy.OVERRIDE)


def print_vertex_unification_result(test_file: str):
    """Print vertex unification result

    Usage:
        print_vertex_unification_result("schema.input")
    """
    if not test_file:
        return
    filename = str(pathlib.Path(__file__).parent.joinpath(_DIR_PATH, test_file))
    module = ParseFile(filename)
    vertex, unify_vertex, merged_module = MergeASTToVertex(module)
    print("Origin vertex formed by AST as follows:\n")
    print(vertex.pretty())
    print("Unify vertex result as follows:\n")
    print(unify_vertex.pretty())
    print("Merged AST as follows:\n")
    print(merged_module.to_json())


if __name__ == "__main__":
    unittest.main(verbosity=2)
