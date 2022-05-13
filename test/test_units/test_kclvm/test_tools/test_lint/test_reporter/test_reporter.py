import unittest
import pathlib
import sys
import os

import kclvm.tools.lint.lint.KCLLint as KCLLint
from kclvm.tools.lint.reporters.stdout_reporter import color
from kclvm.tools.lint.message.message import Message

_FILE_INPUT_SUFFIX = ".k"
_FILE_OUTPUT_SUFFIX = ".golden"
_FILE_TEST_OUTPUT_SUFFIX = ".test"
_PATH_NAME = "test_data"
_DIR_PATH = pathlib.Path(__file__).parent.joinpath(_PATH_NAME)

_TEST_CASE_NAMES = [
    "stdout",
    "file"
]

_TEST_CASE = {
    "stdout": {
        "msg": [
            Message("E0401",
                    str(_DIR_PATH.joinpath("reporter.k")),
                    "Unable to import abc.",
                    "import abc as abc",
                    (1, 1),
                    ["abc"]),
        ],
        "msgs_map": {"E0401": 1},
        "expected": {
            "msg_with_color": color(str(_DIR_PATH.joinpath("reporter.k")), "FILE_NAME")
                              + f''':{color("1", "LINE_COLUMN")}:{color("1", "LINE_COLUMN")}: {color("E0401", "ID")} Unable to import abc.
import abc as abc
{color("^", "MARK")}

Check total {color("1", "NUMBER")} files:
{color("1", "NUMBER")}       {color("E0401", "ID")}: Unable to import
KCL Lint: {color("1", "NUMBER")} problems
''',
            "msg_without_color": str(_DIR_PATH.joinpath("reporter.k"))
                                 + f''':1:1: E0401 Unable to import abc.
import abc as abc
^

Check total 1 files:
1       E0401: Unable to import
KCL Lint: 1 problems
'''}},
    "file":
        {"msg": [
            Message("E0401",
                    str(_DIR_PATH.joinpath("reporter.k")),
                    "Unable to import abc.",
                    "import abc as abc",
                    (1, 1),
                    ["abc"]),
        ],
            "msgs_map": {"E0401": 1},
            "expected": ""
        },
    "sarif":
        {"msg": [
            Message("E0401",
                    str(_DIR_PATH.joinpath("reporter.k")),
                    "Unable to import abc.",
                    "import abc as abc",
                    (1, 1),
                    ["abc"]),
        ],
            "msgs_map": {"E0401": 1},
            "expected": ""
        },
}
DEFAULT_CONFIG = {
    "check_list": ["import", "misc"],
    "ignore": [],
    "max_line_length": 200,
    "output": ["stdout"],
    "output_path": None
}
MSGS = {
    "E0401": (
        "Unable to import %s.",
        "Unable to import",
        "Unable to import '{0}'."
    )
}


class KCLReporterBaseTest(unittest.TestCase):
    def setUp(self):
        self.maxDiff = None

    def read_data(self, data_name: str):
        """Read test data"""
        input_filename = data_name + _FILE_INPUT_SUFFIX
        input_file = _DIR_PATH / input_filename
        return input_file

    def build_reporter(self, case):
        input_file = self.read_data("reporter")
        linter = KCLLint.KCLLinter(input_file, config=DEFAULT_CONFIG)
        linter.msgs = _TEST_CASE[case]["msg"]
        linter.msgs_map = _TEST_CASE[case]["msgs_map"]
        if case == "file" or case == "sarif":
            linter.config.output_path = str(_DIR_PATH.joinpath(case + "_reporter" + _FILE_TEST_OUTPUT_SUFFIX))
        reporter = KCLLint.ReporterFactory.get_reporter(case, linter)
        linter.MSGS = MSGS
        return reporter


class KCLReporterTest(KCLReporterBaseTest):
    def test_stdout_reporter(self) -> None:
        class _redirect:
            content = ""
            is_atty = False

            def write(self, in_str):
                self.content += in_str

            def flush(self):
                self.content = ""

            def isatty(self):
                return self.is_atty

            def getvalue(self):
                return self.content

        r = _redirect()
        sys.stdout = r
        reporter = self.build_reporter("stdout")
        reporter.print_msg(reporter.linter.msgs, r)
        self.assertEqual(r.content, _TEST_CASE["stdout"]["expected"]["msg_without_color"])

        """
        Print msg with color, remove it temporarily.
        r.is_atty = True
        r.content = ""
        reporter.print_msg(reporter.linter.msgs, r)
        self.assertEqual(r.content, _TEST_CASE["stdout"]["expected"]["msg_with_color"])
        """

    def test_file_reporter(self) -> None:
        reporter = self.build_reporter("file")
        reporter.display()
        test_output = str(_DIR_PATH.joinpath("file_reporter" + _FILE_TEST_OUTPUT_SUFFIX))
        golden_output = str(_DIR_PATH.joinpath("file_reporter" + _FILE_OUTPUT_SUFFIX))
        with open(golden_output) as f:
            golden_output_list = f.readlines()
        golden_output_list[0] = str(_DIR_PATH.joinpath("reporter" + _FILE_INPUT_SUFFIX)) + golden_output_list[0]
        with open(test_output) as f:
            test_output_list = f.readlines()
            test_output_list[0] = str(
                _DIR_PATH.joinpath("reporter" + _FILE_INPUT_SUFFIX)) + ":1:1: E0401 Unable to import abc.\n"
        self.assertListEqual(test_output_list, golden_output_list)
        os.remove(test_output)

    def test_sarif_reporter(self) -> None:
        reporter = self.build_reporter("sarif")
        reporter.display()
        test_output = str(_DIR_PATH.joinpath("sarif_reporter" + _FILE_TEST_OUTPUT_SUFFIX))
        golden_output = str(_DIR_PATH.joinpath("sarif_reporter" + _FILE_OUTPUT_SUFFIX))
        with open(golden_output) as f:
            golden_output_list = f.readlines()
        golden_output_list[10] = "                  \"uri\": \"" + str(
            _DIR_PATH.joinpath("reporter" + _FILE_INPUT_SUFFIX)) + "\""
        with open(test_output) as f:
            test_output_list = f.readlines()
            test_output_list[10] = "                  \"uri\": \"" + str(
                _DIR_PATH.joinpath("reporter" + _FILE_INPUT_SUFFIX)) + "\""
        self.assertListEqual(test_output_list, golden_output_list)
        os.remove(test_output)


if __name__ == "__main__":
    unittest.main(verbosity=2)
