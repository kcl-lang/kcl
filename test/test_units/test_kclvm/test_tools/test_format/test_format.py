#! /usr/bin/env python3

import io
import sys
import unittest
import pathlib
from typing import Tuple

from kclvm.tools.format import kcl_fmt_source, kcl_fmt, TextAdapterWalker, Formatter


_FILE_INPUT_SUFFIX = ".input"
_FILE_OUTPUT_SUFFIX = ".golden"
_PATH_NAME = "format_data"
_DIR_PATH = pathlib.Path(__file__).parent.joinpath(_PATH_NAME)
_TEST_CASES = [
    "assert",
    "check",
    "blankline",
    "breakline",
    "codelayout",
    "collection_if",
    "comment",
    "comp_for",
    "empty",
    "import",
    "indent",
    "inline_comment",
    "lambda",
    "quant",
    "schema",
    "string",
    "type_alias",
    "unary",
]
TEST_FMT_PATH = pathlib.Path(__file__).parent.joinpath("format_path_data")
FMT_PATH_EXPECTED_OUTPUT = pathlib.Path(__file__).parent.joinpath("format_path_data/output.golden").read_text()


class KCLBaseFormatterTest(unittest.TestCase):
    """
    KCL base formatter test
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

    def assert_format_equal(self, data_name: str) -> None:
        """
        Read format test data according to data name and assert equal.
        """
        data_input, data_output = self.read_data(data_name)
        formatted, _ = kcl_fmt_source(data_input)
        self.assertMultiLineEqual(formatted, data_output)

    def assert_printer_equal(self, string: str, walker: TextAdapterWalker) -> bool:
        """
        Read walker printer buffer string and assert it is equal to 'string'
        """
        walker.printer.seek(0)
        self.assertEqual(string, walker.printer.read())


class KCLFormatterTest(KCLBaseFormatterTest):
    def test_format_data(self) -> None:
        """
        Test format data for the comparison of input and golden files
        """
        self.maxDiff = None
        for case in _TEST_CASES:
            print(case+"-begin")
            self.assert_format_equal(case)
            print(case+"-end")

    def test_text_adapter_walker_write(self) -> None:
        """
        Test TextAdapterWalker class write function in kclvm format.py
        """
        inputs = ["test", "123", "456", "\n"]
        outputs = ["test", "test123", "test123456", "test123456\n"]
        walker = TextAdapterWalker()
        for input, output in zip(inputs, outputs):
            walker.write(input)
            self.assert_printer_equal(output, walker)

    def test_text_adapter_walker_blankline(self) -> None:
        """
        Test TextAdapterWalker class blankline function in kclvm format.py
        """
        inputs = [
            "\n",
            "\n\n",
            "\n\n\n",
            "\n \n \n",
            "\n # comment \n \n",
            "\n # comment \n # comment \n",
        ]
        outputs = [0, 1, 2, 2, 1, 0]
        walker = TextAdapterWalker()
        for input, output in zip(inputs, outputs):
            self.assertEqual(walker.count_blank_line(input), output)

    def test_text_adapter_walker_indent(self) -> None:
        """
        Test TextAdapterWalker class indent function in kclvm format.py
        """
        inputs = ["", "    ", "\n  ", "  \n  ", "\n    ", "\n\n    ", "\n  ", "\n"]
        outputs = [0, 0, 2, 2, 4, 4, 2, 0]
        levels = [0, 0, 1, 1, 2, 2, 1, 0]
        walker = TextAdapterWalker()
        for input, output, level in zip(inputs, outputs, levels):
            self.assertEqual(walker.count_indent(input), output)
            self.assertEqual(walker.indent_level, level)

    def test_formatter_split_newline(self) -> None:
        """
        Test Formatter class split_newline function in kclvm format.py
        """
        inputs = [
            "\n# comment\n",
            "\n # comment \n",
            "\n # comment  \n ",
            "\n # comment #  \n",
            "\n\n # comment \\n #  \n",
            "\n\n # comment \\n #  \n#comment\n",
            "\n \n # comment \n # comment \n\n # comment \n",
            "\n\n # comment \\n # # \n # \n # \n \n# comment\n\n# comment \n",
        ]
        outputs = [
            ["\n", "# comment", "\n"],
            ["\n ", "# comment ", "\n"],
            ["\n ", "# comment  ", "\n "],
            ["\n ", "# comment #  ", "\n"],
            ["\n\n ", "# comment \\n #  ", "\n"],
            ["\n\n ", "# comment \\n #  ", "\n", "#comment", "\n"],
            ["\n \n ", "# comment ", "\n ", "# comment ", "\n\n ", "# comment ", "\n"],
            [
                "\n\n ",
                "# comment \\n # # ",
                "\n ",
                "# ",
                "\n ",
                "# ",
                "\n \n",
                "# comment",
                "\n\n",
                "# comment ",
                "\n",
            ],
        ]
        formatter = Formatter()
        for input, output in zip(inputs, outputs):
            self.assertListEqual(formatter.split_newline_value(input), output)

    def test_formatter_write_newline(self) -> None:
        """
        Test Formatter class write newline function in kclvm format.py
        """
        inputs = ["\n", "\n\n", "\n\n\n", "\n\n\n\n"]
        outputs = ["\n", "\n\n", "\n\n", "\n\n"]
        for input, output in zip(inputs, outputs):
            formatter = Formatter()
            formatter.write_newline(input)
            self.assert_printer_equal(output, formatter)

    def test_formatter_write_string(self) -> None:
        """
        Test Formatter class write newline function in kclvm format.py
        """
        inputs = ["'test'", '"test"', "R'test'", 'B"test"']
        outputs = ["'test'", '"test"', "r'test'", 'b"test"']
        for input, output in zip(inputs, outputs):
            formatter = Formatter()
            formatter.write_string(input)
            self.assert_printer_equal(output, formatter)

    def test_kcl_fmt_recursively(self):
        with io.StringIO() as buf:
            origin_io = sys.stdout
            sys.stdout = buf
            kcl_fmt(TEST_FMT_PATH, is_stdout=True, recursively=True)
            # Format two files recursively
            self.assertEqual(buf.getvalue().count(FMT_PATH_EXPECTED_OUTPUT), 2)
            sys.stdout = origin_io

    def test_kcl_fmt_non_recursively(self):
        # Format one file non-recursively
        with io.StringIO() as buf:
            origin_io = sys.stdout
            sys.stdout = buf
            kcl_fmt(TEST_FMT_PATH, is_stdout=True, recursively=False)
            # only one file formatted
            self.assertEqual(buf.getvalue().count(FMT_PATH_EXPECTED_OUTPUT), 1)
            sys.stdout = origin_io

    def test_kcl_fmt_single_file(self):
        # Format single file
        for case in _TEST_CASES:
            with io.StringIO() as buf:
                origin_io = sys.stdout
                sys.stdout = buf
                input_filename = str(_DIR_PATH / (case + _FILE_INPUT_SUFFIX))
                output_filename = case + _FILE_OUTPUT_SUFFIX
                expect_output = (_DIR_PATH / output_filename).read_text()

                kcl_fmt(input_filename, is_stdout=True)
                self.assertEqual(expect_output, buf.getvalue())
                sys.stdout = origin_io


    def test_kcl_fmt_not_stdout(self):
        for case in _TEST_CASES:
            input_filepath = _DIR_PATH / (case + _FILE_INPUT_SUFFIX)
            output_filepath = _DIR_PATH / (case + _FILE_OUTPUT_SUFFIX)
            ori_content_backup = input_filepath.read_text()
            expect_output = output_filepath.read_text()

            kcl_fmt(input_filepath, is_stdout=False)
            formatted_content = input_filepath.read_text()
            self.assertEqual(expect_output, formatted_content)
            input_filepath.write_text(ori_content_backup)


if __name__ == "__main__":
    unittest.main(verbosity=2)
