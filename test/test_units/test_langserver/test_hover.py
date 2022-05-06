import pathlib
import unittest
import typing

from pygls.lsp.types.basic_structures import MarkupContent, MarkupKind, Range, Position
import pygls.lsp.types.language_features.hover as pygls_hover

import kclvm.kcl.ast as ast
import kclvm.tools.langserver.hover as hover
import kclvm.tools.langserver.go_to_def as go_to_def


_TEST_PATH_NAME = "test_data"
_HOVER_PATH_NAME = "hover"
_DIR_PATH = (
    pathlib.Path(__file__).parent.joinpath(_TEST_PATH_NAME).joinpath(_HOVER_PATH_NAME)
)


class HoverTestCase:
    def __init__(
        self,
        filename: str,
        line: int,
        column: int,
        msg: str = None,
        range: typing.Optional[Range] = None,
        no_result: bool = False,
    ):
        self.filename: str = filename
        self.line: int = line
        self.column: int = column
        self.msg: str = msg
        self.range: Range = range
        self.no_result: bool = no_result


class HoverTest(unittest.TestCase):
    test_cases = [
        HoverTestCase(
            filename="incomplete.k",
            line=4,
            column=23,
            msg=f"ProviderConfig\ntype: ProviderConfig\ndefined in:{_DIR_PATH}/incomplete.k",
            range=Range(
                start=Position(line=3, character=15),
                end=Position(line=3, character=29),
            ),
        ),
        HoverTestCase(
            filename="built_in.k",
            line=28,
            column=5,
            msg="(built-in) max(): any\nWith a single iterable argument, return its biggest item.\n    The default keyword-only argument specifies an object to return\n    if the provided iterable is empty. With two or more arguments,\n    return the largest argument.\n    ",
            range=Range(
                start=Position(line=27, character=4),
                end=Position(line=27, character=7),
            ),
        ),
        HoverTestCase(
            filename="built_in.k",
            line=33,
            column=11,
            msg="(built-in) typeof(x: any, full_name: bool=False): str\nReturn the type of the kcl object",
            range=Range(
                start=Position(line=32, character=8),
                end=Position(line=32, character=14),
            ),
        ),
        HoverTestCase(
            filename="built_in.k",
            line=36,
            column=5,
            msg='(built-in) option(key: str, type: str="", required: bool=False, default: any=None, help: str="", help: str="", file: str="", line: int=0): any\nReturn the top level argument by the key',
            range=Range(
                start=Position(line=35, character=4),
                end=Position(line=35, character=10),
            ),
        ),
        HoverTestCase(
            filename="hello.k",
            line=8,
            column=5,
            msg=f"image\ntype: str\ndefined in:{_DIR_PATH}/hello.k",
            range=Range(
                start=Position(line=7, character=4),
                end=Position(line=7, character=9),
            ),
        ),
        HoverTestCase(
            filename="hello.k",
            line=12,
            column=15,
            msg=f"name\ntype: str\ndefined in:{_DIR_PATH}/hello.k",
            range=Range(
                start=Position(line=11, character=13),
                end=Position(line=11, character=17),
            ),
        ),
        HoverTestCase(
            filename="hello.k",
            line=7,
            column=11,
            no_result=True,
        ),
        HoverTestCase(
            filename="hello.k",
            line=5,
            column=1,
            no_result=True,
        ),
        HoverTestCase(
            filename="import.k",
            line=1,
            column=10,
            msg="hello",
            range=Range(
                start=Position(line=0, character=8),
                end=Position(line=0, character=13),
            ),
        ),
    ]

    def test_hover(self):
        for i, t_case in enumerate(self.test_cases):
            if i != 0:
                return
            got = hover.hover(
                pos=ast.Position(
                    filename=str(_DIR_PATH / t_case.filename),
                    line=t_case.line,
                    column=t_case.column,
                ),
            )
            expect = (
                pygls_hover.Hover(
                    contents=MarkupContent(
                        kind=MarkupKind.PlainText,
                        value=t_case.msg,
                    ),
                    range=t_case.range,
                )
                if not t_case.no_result
                else None
            )
            self.assertEqual(
                expect,
                got,
                f"err hover for case[{i}], from pos:{t_case.filename}:{t_case.line}:{t_case.column}\nexpect: {expect}, got: {got}",
            )


class NoneInputTest(unittest.TestCase):
    def test_definition_None_input(self):
        node, scope_obj = go_to_def.definition(None, None)
        self.assertIsNone(node)
        self.assertIsNone(scope_obj)

    def test_scope_obj_desc_None_input(self):
        result = hover.scope_obj_desc(None, None)
        self.assertIsNone(result)
