import unittest
import pathlib
import kclvm.tools.lint.lint.KCLLint as KCLLint
from kclvm.tools.lint.message.message import Message
import kclvm.compiler.parser.parser as parser

_FILE_INPUT_SUFFIX = ".k"
_FILE_OUTPUT_SUFFIX = ".golden"
_PATH_NAME = "test_data"
_DIR_PATH = pathlib.Path(__file__).parent.joinpath(_PATH_NAME)
_TEST_CASE_NAMES = [
    "import",
    "misc",
    "basic",
]
_TEST_CASE_OUTPUT = {
    "import": [
        Message("E0401",
                str(_DIR_PATH.joinpath("import.k")),
                "Unable to import abc.",
                "import abc # unable to import",
                (1, 1),
                ["abc"]),
        Message("W0404",
                str(_DIR_PATH.joinpath("import.k")),
                "a is reimported multiple times.",
                "import a # reimport",
                (3, 1),
                ["a"]),
        Message("E0413",
                str(_DIR_PATH.joinpath("import.k")),
                "Import b should be placed at the top of the module.",
                "import b # import position",
                (18, 1),
                ["b"]),
        Message("W0411",
                str(_DIR_PATH.joinpath("import.k")),
                "math imported but unused.",
                "import math",
                (5, 1),
                ["math"]),
        Message("W0411",
                str(_DIR_PATH.joinpath("import.k")),
                "b imported but unused.",
                "import b # import position",
                (18, 1),
                ["b"]),

    ],
    "misc": [
        Message("E0501",
                str(_DIR_PATH.joinpath("misc.k")),
                "line too long (121 > 120 characters).",
                "# line too long, line too long, line too long, line too long, line too long, line too long, line too long, line too long,",
                (1, 1),
                [121, 120])
    ],
    "basic": [
        Message("C0103",
                str(_DIR_PATH.joinpath("basic.k")),
                "Variable name \"I\" doesn't conform to ^[^\W\dA-Z][^\W_]+$.",
                "I = 1",
                (2, 1),
                ['Variable', 'I', '^[^\\W\\dA-Z][^\\W_]+$']),

        Message("C0104",
                str(_DIR_PATH.joinpath("basic.k")),
                "Disallowed name \"I\".",
                "I = 1",
                (2, 1),
                ['I']),
        Message("C0103",
                str(_DIR_PATH.joinpath("basic.k")),
                "Schema name \"person\" doesn't conform to PascalCase naming style.",
                "schema person:",
                (5, 8),
                ['Schema', 'person', 'PascalCase naming style']),
        Message("C0103",
                str(_DIR_PATH.joinpath("basic.k")),
                "Mixin name \"personMixin\" doesn't conform to PascalCase naming style.",
                "mixin personMixin:",
                (9, 7),
                ['Mixin', 'personMixin', 'PascalCase naming style']),
        Message("C0103",
                str(_DIR_PATH.joinpath("basic.k")),
                "Argument name \"PaP\" doesn't conform to camelCase naming style.",
                "schema Person[PaP]:",
                (13, 15),
                ['Argument', 'PaP', 'camelCase naming style']),
        Message("C0103",
                str(_DIR_PATH.joinpath("basic.k")),
                "Schema attribute name \"Age\" doesn't conform to camelCase naming style.",
                "    Age : int = 1",
                (15, 5),
                ['Schema attribute', 'Age', 'camelCase naming style']),
        Message("C0103",
                str(_DIR_PATH.joinpath("basic.k")),
                "Variable name \"pers_on\" doesn't conform to ^[^\W\dA-Z][^\W_]+$.",
                "pers_on = Person {",
                (19, 1),
                ['Variable', 'pers_on', '^[^\\W\\dA-Z][^\\W_]+$']),
        Message("C0103",
                str(_DIR_PATH.joinpath("basic.k")),
                "Protocol name \"personProtocol\" doesn't conform to PascalCase naming style.",
                "protocol personProtocol:",
                (24, 10),
                ['Protocol', 'personProtocol', 'PascalCase naming style']),
        Message("C0103",
                str(_DIR_PATH.joinpath("basic.k")),
                "Schema name \"someRule\" doesn't conform to PascalCase naming style.",
                "rule someRule:",
                (39, 6),
                ['Schema', 'someRule', 'PascalCase naming style']),
    ],
}
CHECKER_CONFIG = {
    "check_list": ["import", "misc", "basic"],
    "ignore": [],
    "max_line_length": 120,
    "output": ["stdout"],
    "output_path": None,
    "module_naming_style": "ANY",
    "package_naming_style": "ANY",
    "schema_naming_style": "PascalCase",
    "mixin_naming_style": "PascalCase",
    "argument_naming_style": "camelCase",
    "schema_attribute_naming_style": "camelCase",
    "variable_rgx": r"^[^\W\dA-Z][^\W_]+$",
    "bad_names": ["foo", "bar", "baz", "toto", "tata", "tutu", "I", "l", "O"],
}


class KCLCheckerBaseTest(unittest.TestCase):
    def setUp(self):
        self.maxDiff = None

    def read_data(self, data_name: str):
        """Read test data"""
        input_filename = data_name + _FILE_INPUT_SUFFIX
        input_file = _DIR_PATH / input_filename
        with open(input_file) as f:
            code = f.read()

        return input_file, code

    def assert_checker_msgs_equal(
            self, case: str, input_file: str, code: str
    ):
        checker = KCLLint.CheckerFactory.get_checker(case)
        checker.options = KCLLint.LinterConfig()
        checker.options.update(CHECKER_CONFIG)
        prog = parser.LoadProgram(input_file)
        checker.check(prog, code)
        for i, m in enumerate(checker.msgs):
            expect = _TEST_CASE_OUTPUT[case][i]
            self.assertEqual(expect, m, f"Expected:\n{expect}\narguments:{expect.arguments}\nActual:\n{m}\narguments:{m.arguments}")


class KCLCheckerTest(KCLCheckerBaseTest):
    def test_checker_msgs(self) -> None:
        for case in _TEST_CASE_NAMES:
            input_file, code = self.read_data(case)
            self.assert_checker_msgs_equal(case, input_file, code)

    def test_dot_path(self) -> None:
        # When the path is `.`, the filename in ast is "a/./b". This may affect import path check, e.g: a/./b == a/b
        input_file = _DIR_PATH / ("./import" + _FILE_INPUT_SUFFIX)
        with open(input_file) as f:
            code = f.read()
        checker = KCLLint.CheckerFactory.get_checker("import")
        checker.options = CHECKER_CONFIG
        prog = parser.LoadProgram(input_file)
        checker.check(prog, code)


if __name__ == "__main__":
    unittest.main(verbosity=2)
