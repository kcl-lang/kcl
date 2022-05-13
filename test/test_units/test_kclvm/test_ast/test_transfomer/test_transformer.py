#! /usr/bin/env python3

import io
import unittest
import pathlib
from copy import deepcopy
from typing import List

from kclvm.compiler.parser import ParseFile
from kclvm.tools.printer import PrintAST
import kclvm.kcl.ast as ast


_FILE_INPUT_SUFFIX = ".input"
_FILE_OUTPUT_SUFFIX = ".output"
_PATH_NAME = "test_data"
_DIR_PATH = pathlib.Path(__file__).parent.joinpath(_PATH_NAME)


class TestTransformer(ast.TreeTransformer):
    def walk_AssignStmt(self, t: ast.AssignStmt):
        name = t.targets[0].get_name()
        t.targets.append(
            ast.Identifier(
                line=t.line,
                column=t.column,
                names=[name * 2],
                ctx=ast.ExprContext.STORE,
            )
        )
        return t


class AssignSplitTransformer(ast.TreeTransformer):
    def walk_AssignStmt(self, t: ast.AssignStmt):
        if len(t.targets) > 1:
            assign_stmt_list: List[ast.AssertStmt] = []
            for target in t.targets:
                t_copy = deepcopy(t)
                t_copy.targets = [target]
                assign_stmt_list.append(t_copy)
            return assign_stmt_list
        return t


class KCLBaseTreeTransformerTest(unittest.TestCase):
    """KCL AST transfomer test"""

    def setUp(self):
        inputs = list(sorted(pathlib.Path(_DIR_PATH).glob("*" + _FILE_INPUT_SUFFIX)))
        case_inputs = [input.read_text() for input in inputs]
        outputs = list(sorted(pathlib.Path(_DIR_PATH).glob("*" + _FILE_OUTPUT_SUFFIX)))
        case_outputs = [output.read_text() for output in outputs]
        names = [str(output.with_suffix("").name) for output in outputs]
        self.TEST_CASES = zip(names, case_inputs, case_outputs)
        return super().setUp()

    def transform_code(self, code: str, transformer: ast.TreeTransformer) -> str:
        module = ParseFile("", code)
        transformer().walk(module)
        with io.StringIO() as buf:
            PrintAST(module, buf)
            return buf.getvalue()


class KCLTreeTransformerTest(KCLBaseTreeTransformerTest):
    """KCL AST transfomer test"""

    transformer_mapping = {
        "assign": TestTransformer,
        "assign_split": AssignSplitTransformer,
    }

    def test_transform(self):
        for name, case_input, case_output in self.TEST_CASES:
            transformer = self.transformer_mapping.get(name, TestTransformer)
            self.assertEqual(self.transform_code(case_input, transformer), case_output)


if __name__ == "__main__":
    unittest.main(verbosity=2)
