import os
import sys
import unittest
import pathlib
import subprocess
from typing import Tuple
import kclvm.kcl.ast as ast
from kclvm.compiler.parser import ParseFile
from kclvm.tools.list_attribute.utils import ListAttributePrinter

_FILE_INPUT_SUFFIX = ".k"
_FILE_OUTPUT_SUFFIX = ".golden"
_FILE_FULLSCHEMA_SUFFIX = ".full_schema"
_PATH_NAME = "test_data"
_DIR_PATH = pathlib.Path(__file__).parent.joinpath(_PATH_NAME)
_TEST_CASE_NAMES = [
    "list_attr"
]


class KCLListAttrBaseTest(unittest.TestCase):
    def setUp(self):
        self.maxDiff = None

    def read_data(self, data_name: str):
        """Read test data"""
        input_filename = data_name + _FILE_INPUT_SUFFIX
        output_filename = data_name + _FILE_OUTPUT_SUFFIX
        full_schema_filename = data_name + _FILE_FULLSCHEMA_SUFFIX
        input_file = _DIR_PATH / input_filename
        data_input = (_DIR_PATH / input_filename).read_text()
        data_output = (_DIR_PATH / output_filename).read_text()
        data_full_schema = (_DIR_PATH / full_schema_filename).read_text()
        return input_file, data_input, data_output, data_full_schema

    def assert_full_schema_equal(
            self,
            data_name: str
    ):
        input_file, input_str, output_str, full_schema_str = self.read_data(data_name)
        printer = ListAttributePrinter(str(input_file))
        printer.build_full_schema_list()
        if not printer.full_schema_list:
            return
        full_schema_list = printer.full_schema_list
        full_schema = ""
        for s in full_schema_list:
            full_schema += str(s) + "\n"
        full_schema = full_schema[:-1]
        self.assertEqual(full_schema, full_schema_str)

    def assert_list_attr_equal(
            self,
            data_name: str
    ):
        input_file, input_str, output_str, full_schema_str = self.read_data(data_name)
        buffer = Redirect()
        current = sys.stdout
        sys.stdout = buffer
        ListAttributePrinter(str(input_file)).print()
        sys.stdout = current
        self.assertEqual(buffer.content, output_str)


# rewrite sys.stdout.write() to get content of print()
class Redirect:
    def __init__(self):
        self.content = ""

    def write(self, str):
        self.content += str


class KCLListAttrTest(KCLListAttrBaseTest):
    def test_full_schema(self) -> None:
        for case in _TEST_CASE_NAMES:
            self.assert_full_schema_equal(case)

    def test_list_attr_schema(self) -> None:
        for case in _TEST_CASE_NAMES:
            self.assert_list_attr_equal(case)


if __name__ == "__main__":
    unittest.main(verbosity=2)
