#! /usr/bin/env python3

import os
import io
import unittest
import pathlib
from typing import Tuple

from kclvm.compiler.parser import ParseFile, ParseMode


_FILE_INPUT_SUFFIX = ".input"
_FILE_OUTPUT_SUFFIX = ".output"
_PATH_NAME = "test_data"
_DIR_PATH = pathlib.Path(__file__).parent.joinpath(_PATH_NAME)
_TEST_CASES = [
    "arguments",
    "empty",
    "if_stmt",
    "import",
    "codelayout",
    "collection_if",
    "comment",
    "index_sign",
    "joined_str",
    "lambda",
    "quant",
    "rule",
    "str",
    "type_alias",
    "unary",
    "unification",
]


class KCLBasePrinterTest(unittest.TestCase):
    """
    KCL base Printer test
    """

    def read_data(self, data_name: str) -> Tuple[str, str]:
        """
        Read printer data according to data name
        """
        input_filename = data_name + _FILE_INPUT_SUFFIX
        output_filename = data_name + _FILE_OUTPUT_SUFFIX
        data_input = (_DIR_PATH / input_filename).read_text()
        data_output = (_DIR_PATH / output_filename).read_text()
        return data_input, data_output

    def assert_printer_equal(self, data_name: str) -> None:
        """
        Read printer test data according to data name and assert equal.
        """
        from kclvm.tools.printer import PrintAST

        data_input, data_output = self.read_data(data_name)
        module = ParseFile("", data_input, mode=ParseMode.ParseComments)
        with io.StringIO() as str_io:
            PrintAST(module, str_io)
            self.assertEqual(str_io.getvalue(), data_output)


class KCLPrinterTest(KCLBasePrinterTest):
    def test_printer_data(self) -> None:
        """
        Test printer data for the comparison of input and golden files
        """
        self.maxDiff = None
        for case in _TEST_CASES:
            self.assert_printer_equal(case)


if __name__ == "__main__":
    unittest.main(verbosity=2)
