import pathlib
import unittest

import kclvm.tools.lint.lint.KCLLint as KCLLint
import kclvm.tools.lint.lint.exceptions as exceptions


LINT_CONFIG_SUFFIX = ".kcllint"
_PATH_NAME = "test_data"
_FILE_NAME = "main.k"
_DIR_PATH = pathlib.Path(__file__).parent.joinpath(_PATH_NAME)
_TEST_FILE = _DIR_PATH.joinpath(_FILE_NAME)
_WRONG_FILE_NAME = "mian.k"
_WRONG_TEST_FILE = _DIR_PATH.joinpath(_WRONG_FILE_NAME)


TEST_CASE = {
    "invalid_checker": {
        "code": "",
        "path": _TEST_FILE,
        "config": {
            "check_list": ["invalid_checker"],
            "ignore": [],
            "max_line_length": 200,
            "output": ["stdout"],
        }
    },
    "invalid_reporter": {
        "code": "",
        "path": _TEST_FILE,
        "config": {
            "check_list": ["import"],
            "ignore": [],
            "max_line_length": 200,
            "output": ["invalid_reporter"],
        }
    },
    "empty_reporter": {
        "code": "",
        "path": _TEST_FILE,
        "config": {
            "output": [],
        }
    },
    "without_output_path": {
        "code": "",
        "path": _TEST_FILE,
        "config": {
            "output": ["file"],
        }
    },
    "wrong_path": {
        "path": _WRONG_TEST_FILE,
    }
}


class KCLLintExceptionTest(unittest.TestCase):
    def test_invalid_checker(self):
        code = TEST_CASE["invalid_checker"]["code"]
        config = TEST_CASE["invalid_checker"]["config"]
        path = TEST_CASE["invalid_checker"]["path"]
        try:
            KCLLint.kcl_lint_code(path, k_code_list=[code], config=config)
            assert False
        except Exception as err:
            assert isinstance(err, exceptions.InvalidCheckerError)

    def test_invalid_reporter(self):
        code = TEST_CASE["invalid_reporter"]["code"]
        config = TEST_CASE["invalid_reporter"]["config"]
        path = TEST_CASE["invalid_reporter"]["path"]
        try:
            KCLLint.kcl_lint_code(path, k_code_list=[code], config=config)
            assert False
        except Exception as err:
            assert isinstance(err, exceptions.InvalidReporterError)

    def test_empty_reporter(self):
        code = TEST_CASE["empty_reporter"]["code"]
        config = TEST_CASE["empty_reporter"]["config"]
        path = TEST_CASE["empty_reporter"]["path"]
        try:
            KCLLint.kcl_lint_code(path, k_code_list=[code], config=config)
            assert False
        except Exception as err:
            assert isinstance(err, exceptions.EmptyReporterError)

    def test_without_output_path(self):
        code = TEST_CASE["without_output_path"]["code"]
        config = TEST_CASE["without_output_path"]["config"]
        path = TEST_CASE["without_output_path"]["path"]
        try:
            KCLLint.kcl_lint_code(path, k_code_list=[code], config=config)
            assert False
        except Exception as err:
            assert isinstance(err, AssertionError)
            self.assertEqual(err.args, ("Without ouput file path",))

    def test_wrong_path(self):
        path = TEST_CASE["wrong_path"]["path"]
        try:
            KCLLint.kcl_lint(path)
            assert False
        except Exception as err:
            assert isinstance(err, FileNotFoundError)

if __name__ == "__main__":
    unittest.main(verbosity=2)
