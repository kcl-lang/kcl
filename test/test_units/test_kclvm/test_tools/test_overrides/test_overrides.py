#! /usr/bin/env python3

import io
import os
import unittest
import pathlib

from typing import Tuple, List, Union

import kclvm.kcl.ast as ast
import kclvm.kcl.error as kcl_error
from kclvm.compiler.parser import ParseFile, LoadProgram


FILE_INPUT_SUFFIX = ".input"
FILE_OUTPUT_SUFFIX = ".output"
PATH_NAME = "test_data"
TEST_DIR_PATH = pathlib.Path(__file__).parent.joinpath(PATH_NAME)
CMD_OVERRIDES = [
    ast.CmdOverrideSpec(
        field_path="appConfiguration.image",
        field_value="kusion/kusion:v0.3.1",
    ),
    ast.CmdOverrideSpec(
        field_path="appConfiguration.mainContainer.name",
        field_value="kusion_override",
    ),
    ast.CmdOverrideSpec(
        field_path="appConfiguration.labels.key.key",
        field_value="override_value",
    ),
    ast.CmdOverrideSpec(
        field_path="appConfiguration.overQuota",
        field_value="False",
    ),
    ast.CmdOverrideSpec(
        field_path="appConfiguration.probe",
        field_value="{periodSeconds=20}",
    ),
    ast.CmdOverrideSpec(
        field_path="appConfigurationUnification.image",
        field_value="kusion/kusion:v0.3.1",
    ),
    ast.CmdOverrideSpec(
        field_path="appConfigurationUnification.mainContainer.name",
        field_value="kusion_override",
    ),
    ast.CmdOverrideSpec(
        field_path="appConfigurationUnification.labels.key.key",
        field_value="override_value",
    ),
    ast.CmdOverrideSpec(
        field_path="appConfigurationUnification.overQuota",
        field_value="False",
    ),
    ast.CmdOverrideSpec(
        field_path="appConfigurationUnification.resource",
        action=ast.OverrideAction.DELETE,
    ),
]


def _read_override_test_cases_from_dir(
    dir: Union[pathlib.Path, str]
) -> List[Tuple[str, str, str]]:
    if not dir:
        return []
    inputs = list(sorted(pathlib.Path(dir).glob("*" + FILE_INPUT_SUFFIX)))
    case_inputs = [input.read_text() for input in inputs]
    outputs = list(sorted(pathlib.Path(dir).glob("*" + FILE_OUTPUT_SUFFIX)))
    case_outputs = [output.read_text() for output in outputs]
    filenames = [str(input) for input in inputs]
    return zip(filenames, case_inputs, case_outputs)


class KCLBaseOverrideTest(unittest.TestCase):
    """KCL Override test"""

    def setUp(self):
        self.test_cases = _read_override_test_cases_from_dir(dir=TEST_DIR_PATH)
        self.maxDiff = None
        return super().setUp()

    def assertOverrideEqual(self, filename: str, case_input: str, case_output: str):
        from kclvm.tools.query import ApplyOverrides
        from kclvm.tools.printer import PrintAST

        if not filename or not case_input or not case_output:
            return
        prog = LoadProgram(filename)
        ApplyOverrides(prog, CMD_OVERRIDES)
        with io.StringIO() as out:
            PrintAST(prog.pkgs[prog.main][0], out)
            out.write("\n")
            self.assertEqual(out.getvalue(), case_output)

    def print_overrides_ast(self):
        from kclvm.tools.query import PrintOverridesAST, OverrideInfo

        for i in range(len(OverrideInfo.MODIFIED)):
            OverrideInfo.MODIFIED[i].filename = OverrideInfo.MODIFIED[i].filename.replace(
                FILE_INPUT_SUFFIX,
                FILE_OUTPUT_SUFFIX,
            )
        PrintOverridesAST()
        OverrideInfo.MODIFIED = []


class KCLOverridesTest(KCLBaseOverrideTest):
    """KCL Override test"""

    def test_overrides(self):
        for filename, case_input, case_output in self.test_cases:
            self.assertOverrideEqual(filename, case_input, case_output)

    def test_override_file(self):
        from kclvm.tools.query.override import override_file

        file = str(pathlib.Path(__file__).parent.joinpath("file_test_data").joinpath("test.k"))
        specs = ["config.image=\"image/image\"", ":config.image=\"image/image:v1\"", ":config.data={id=1,value=\"override_value\"}"]
        self.assertEqual(override_file(file, specs), True)

    def test_override_file_auto_fix(self):
        from kclvm.tools.query.override import override_file
        from kclvm.tools.query.override import KCL_FEATURE_GATE_OVERRIDE_AUTO_FIX_ENV

        os.environ[KCL_FEATURE_GATE_OVERRIDE_AUTO_FIX_ENV] = "1"
        file = str(pathlib.Path(__file__).parent.joinpath("file_test_data").joinpath("test_auto_fix.k"))
        specs = ["config.image=\"image/image\"", ":config.image=\"image/image:v1\"", ":config.data={id=1,value=1}"]
        self.assertEqual(override_file(file, specs), True)
        del os.environ[KCL_FEATURE_GATE_OVERRIDE_AUTO_FIX_ENV]

    def test_override_file_auto_schema_import(self):
        from kclvm.tools.query.override import override_file
        from kclvm.tools.query.override import KCL_FEATURE_GATE_OVERRIDE_AUTO_FIX_ENV

        os.environ[KCL_FEATURE_GATE_OVERRIDE_AUTO_FIX_ENV] = "1"
        file = str(pathlib.Path(__file__).parent.joinpath("file_test_data").joinpath("test_auto_import_schema.k"))
        specs = ["x0.data=Data{id=1}"]
        self.assertEqual(override_file(file, specs), True)
        del os.environ[KCL_FEATURE_GATE_OVERRIDE_AUTO_FIX_ENV]

    def test_override_file_with_import_paths(self):
        from kclvm.tools.query.override import override_file

        file = str(pathlib.Path(__file__).parent.joinpath("file_test_data").joinpath("test_import_paths.k"))
        specs = ["data.value=\"override_value\""]
        import_paths = ["pkg", "pkg.pkg"]
        self.assertEqual(override_file(file, specs, import_paths), True)

    def test_override_file_invalid(self):
        from kclvm.tools.query.override import override_file

        specs = [":a:", "a=1", ":a", "a-1"]
        for spec in specs:
            with self.assertRaises(kcl_error.KCLException):
                override_file("main.k", [spec])


if __name__ == "__main__":
    unittest.main(verbosity=2)
