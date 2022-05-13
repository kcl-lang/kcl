# Copyright 2021 The KCL Authors. All rights reserved.

import unittest
import pathlib
from typing import Tuple

import kclvm.api.object as kcl_object
import kclvm.kcl.error as kcl_error
from kclvm.program.eval import EvalCode
from kclvm.vm.code import Opcode
from kclvm.vm.runtime.evaluator.eval import Evaluator

_FILE_INPUT_SUFFIX = ".k"
_FILE_OUTPUT_SUFFIX = ".yaml"
_PATH_NAME = "eval_data"
_DIR_PATH = pathlib.Path(__file__).parent.joinpath(_PATH_NAME)
_TEST_CASES = [
    "assert",
    "aug_assign",
    "calculation",
    "collection_if",
    "compare",
    "convert_collection_value",
    "config",
    "emit_expr",
    "empty",
    "expr",
    "quant_expr",
    "regex",
    "rule",
    "schema_args",
    "schema",
    "type_alias",
    "type_as",
    "str",
    "types",
    "complex",
    "for",
    "if",
    "index_signature",
    "lambda",
    "member_ship",
    "nest_var",
    "plugin",
    "plus",
    "line_continue",
    "list",
    "unary",
    "unification_with_mixin",
    "unification",
    "units",
]


class KCLBaseEvaluatorTest(unittest.TestCase):
    """
    KCL base Evaluator test
    """

    def read_data(self, data_name: str) -> Tuple[str, str]:
        """
        Read format data according to data name
        """
        input_filename = data_name + _FILE_INPUT_SUFFIX
        output_filename = data_name + _FILE_OUTPUT_SUFFIX
        data_input = (_DIR_PATH / input_filename).read_text()
        data_output = (_DIR_PATH / output_filename).read_text()
        return data_input, data_output


class KCLEvaluatorTest(KCLBaseEvaluatorTest):
    def test_Evaluator_eval_binary_op(self):
        kcl_eval = Evaluator()
        self.assertEqual(kcl_object.KCLIntObject(value=11),
                         kcl_eval.eval_binary_op(left=kcl_object.KCLIntObject(value=10),
                                                 right=kcl_object.KCLIntObject(value=11),
                                                 code=Opcode.BINARY_OR))

    def test_eval_data(self) -> None:
        """
        Test format data for the comparison of input and golden files
        """
        self.maxDiff = None
        for case in _TEST_CASES:
            try:
                input_code, output = self.read_data(case)
                self.assertEqual(EvalCode(filename=case+".k", code=input_code), output, msg=f"the case is {case}")
            except Exception as err:
                print(f"case {case} raise an error: {err}")

    def test_invalid_eval_data(self):
        for p in pathlib.Path(__file__).parent.joinpath("invalid_eval_data").glob("*.k"):
            with self.assertRaises(kcl_error.KCLException):
                EvalCode(filename=str(p))


if __name__ == "__main__":
    unittest.main(verbosity=2)
