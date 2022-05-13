import pathlib
import unittest

import kclvm.tools.lint.lint.KCLLint as KCLLint
from kclvm.tools.lint.checkers.imports import ImportsChecker
from kclvm.tools.lint.checkers.misc import MiscChecker
from kclvm.tools.lint.checkers.basic import BasicChecker
from kclvm.tools.lint.reporters.stdout_reporter import STDOUTReporter
from kclvm.tools.lint.message.message import Message

LINT_CONFIG_SUFFIX = ".kcllint"
_FILE_INPUT_SUFFIX = ".k"
_PATH_NAME = "test_data"
_DIR_PATH = pathlib.Path(__file__).parent.joinpath(_PATH_NAME)


class KCLLintIntegrationTest(unittest.TestCase):
    msgs = [
        Message(
            "E0999",
            str(_DIR_PATH.joinpath("failed.k")),
            "Parse failed:InvalidSyntax.",
            "a ==== 1 # Parse failed",
            (1, 5),
            ["InvalidSyntax"]
        ),
        Message(
            "W0411",
            str(_DIR_PATH.joinpath("main.k")),
            "empty_file imported but unused.",
            "import empty_file",
            (1, 1),
            ["empty_file"]
        ),
    ]
    lint_code_test_case = [
        {
            "code": """a ==== 1 # Parse failed
""",
            "file": str(_DIR_PATH.joinpath("failed.k")),
            "msgs": [
                Message(
                    "E0999",
                    str(_DIR_PATH.joinpath("failed.k")),
                    "Parse failed:InvalidSyntax.",
                    "a ==== 1 # Parse failed",
                    (1, 5),
                    ["InvalidSyntax"]
                )
            ],
        },
        {
            "code": """import empty_file
""",
            "file": str(_DIR_PATH.joinpath("main.k")),
            "msgs": [
                Message(
                    "W0411",
                    str(_DIR_PATH.joinpath("main.k")),
                    "empty_file imported but unused.",
                    "import empty_file",
                    (1, 1),
                    ["empty_file"]
                )
            ],
        },
    ]

    def setUp(self) -> None:
        self.linter = KCLLint.KCLLinter(_DIR_PATH)
        self.linter.run()

    def test_load_config(self):
        config = {
            "check_list": ["import", "misc", "basic"],
            "ignore": [],
            "max_line_length": 120,  # default 200, overwrite in .kcllint
            "output": ["stdout"],
            "output_path":None,
            "module_naming_style": "ANY",
            "package_naming_style": "ANY",
            "schema_naming_style": "PascalCase",
            "mixin_naming_style": "PascalCase",
            "argument_naming_style": "camelCase",
            "variable_naming_style": "ANY",
            "schema_attribute_naming_style": "ANY",
            "module_rgx": None,
            "package_rgx": None,
            "schema_rgx": None,
            "mixin_rgx": None,
            "argument_rgx": None,
            "variable_rgx": None,
            "schema_attribute_rgx": None,
            "bad_names": ["foo", "bar", "baz", "toto", "tata", "tutu", "I", "l", "O"],
        }
        for k, v in config.items():
            self.assertEqual(getattr(self.linter.config, k), v)

    def test_register_checkers(self):
        checker_list = [ImportsChecker(self.linter), MiscChecker(self.linter), BasicChecker(self.linter)]
        self.assertListEqual(self.linter.checkers, checker_list)

    def test_register_reporter(self):
        reporter_list = [STDOUTReporter(self.linter)]
        self.assertListEqual(self.linter.reporters, reporter_list)

    def test_ignore_msg(self):
        self.linter.config.ignore = ["E0999", "W0411"]
        self.linter.run()
        self.assertEqual(len(self.linter.msgs), 0)

    def test_kcl_lint_fun(self):
        msgs = KCLLint.kcl_lint(_DIR_PATH)
        self.assertListEqual(msgs, self.msgs)

    def test_kcl_lint_code_fun(self):
        for case in self.lint_code_test_case:
            code = case["code"]
            msgs = KCLLint.kcl_lint_code(case["file"], k_code_list=[code])
            for i, m in enumerate(msgs):
                expect = case["msgs"][i]
                self.assertEqual(expect, m, f"Expected:\n{expect}\narguments:{expect.arguments}\nActual:\n{m}\narguments:{m.arguments}")


if __name__ == "__main__":
    unittest.main(verbosity=2)
