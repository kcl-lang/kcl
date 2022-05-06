import pathlib
import unittest
import typing

from pygls.lsp.types.language_features.completion import (
    CompletionList,
    CompletionItem,
    CompletionItemKind,
)

import kclvm.kcl.ast as ast
from kclvm.tools.langserver.complete import complete


_TEST_PATH_NAME = "test_data"
_COMPLETE_PATH_NAME = "complete"
_DIR_PATH = (
    pathlib.Path(__file__)
    .parent.joinpath(_TEST_PATH_NAME)
    .joinpath(_COMPLETE_PATH_NAME)
)


class CompleteTest(unittest.TestCase):
    def test_complete(self):
        for t_case in test_cases:
            got_completions = complete(
                pos=ast.Position(
                    filename=str(_DIR_PATH / t_case.filename),
                    line=t_case.line,
                    column=t_case.column,
                ),
                name=t_case.name,
            )

            expect_completions = t_case.completions

            self.assertEqual(
                expect_completions,
                got_completions,
                f"expect: {expect_completions}, got: {got_completions}",
            )


class CompletionTestCase:
    def __init__(
        self,
        filename: str,
        line: int,
        column: int,
        name: str,
        completions: typing.List[CompletionItem],
    ):
        self.filename: str = filename
        self.line: int = line
        self.column: int = column
        self.name: str = name
        self.completions: typing.List[CompletionItem] = completions


test_cases = [
    CompletionTestCase(
        filename="simple.k",
        line=2,
        column=6,
        name="a",
        completions=CompletionList(
            is_incomplete=False,
            items=[
                CompletionItem(label="aa", kind=CompletionItemKind.Value),
                CompletionItem(label="abs", kind=CompletionItemKind.Function),
                CompletionItem(label="all_true", kind=CompletionItemKind.Function),
                CompletionItem(label="any_true", kind=CompletionItemKind.Function),
            ],
        ).items,
    ),
    CompletionTestCase(
        filename="schema.k",
        line=4,
        column=8,
        name="Per",
        completions=CompletionList(
            is_incomplete=False,
            items=[
                CompletionItem(label="Person", kind=CompletionItemKind.Struct),
            ],
        ).items,
    ),
]

if __name__ == "__main__":
    unittest.main(verbosity=2)
