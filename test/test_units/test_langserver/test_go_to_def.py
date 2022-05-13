# Copyright 2021 The KCL Authors. All rights reserved.

import pathlib
import unittest

from typing import List
from pygls.lsp.types.basic_structures import Location, Range, Position

import kclvm.kcl.ast as ast
import kclvm.tools.langserver.common as common
import kclvm.tools.langserver.go_to_def as go_to_def


_TEST_PATH_NAME = "test_data"
_GO_TO_DEF_PATH_NAME = "go_to_def"
_DIR_PATH = (
    pathlib.Path(__file__)
    .parent.joinpath(_TEST_PATH_NAME)
    .joinpath(_GO_TO_DEF_PATH_NAME)
)


class GoToDefTestCase:
    def __init__(
        self,
        filename: str,
        line: int,
        column: int,
        locations: List[Location],
    ):
        self.filename: str = filename
        self.line: int = line
        self.column: int = column
        self.locations: List[Location] = locations


class GoToDefTest(unittest.TestCase):
    test_cases = [
        GoToDefTestCase(
            filename="dict_fix_me.k",
            line=15,
            column=9,
            locations=[
                Location(
                    uri="dict_fix_me.k",
                    range=Range(
                        start=Position(line=14, character=8),
                        end=Position(line=14, character=13),
                    ),
                    # range=Range(
                    #     start=Position(line=9, character=4),
                    #     end=Position(line=9, character=9),
                    # ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="dict_fix_me.k",
            line=14,
            column=6,
            locations=[
                Location(
                    uri="dict_fix_me.k",
                    range=Range(
                        start=Position(line=13, character=4),
                        end=Position(line=13, character=8),
                    ),
                    # range=Range(
                    #     start=Position(line=1, character=4),
                    #     end=Position(line=1, character=8),
                    # ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="import_module.k",
            line=3,
            column=11,
            locations=[
                Location(
                    uri="pkg/parent.k",
                    range=Range(
                        start=Position(line=0, character=7),
                        end=Position(line=0, character=13),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="pkg/import_abs.k",
            line=4,
            column=5,
            locations=[
                Location(
                    uri="pkg/parent.k",
                    range=Range(
                        start=Position(line=1, character=4),
                        end=Position(line=1, character=8),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="pkg/import_abs.k",
            line=1,
            column=12,
            locations=[
                Location(
                    uri="pkg/parent.k",
                    range=Range(
                        start=Position(line=0, character=0),
                        end=Position(line=0, character=0),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="simple.k",
            line=2,
            column=5,
            locations=[
                Location(
                    uri="simple.k",
                    range=Range(
                        start=Position(line=0, character=0),
                        end=Position(line=0, character=1),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="schema.k",
            line=3,
            column=17,
            locations=[
                Location(
                    uri="schema.k",
                    range=Range(
                        start=Position(line=1, character=4),
                        end=Position(line=1, character=8),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="import_module.k",
            line=1,
            column=13,
            locations=[
                Location(
                    uri="pkg/parent.k",
                    range=common.emptyRange(),
                )
            ],
        ),
        GoToDefTestCase(
            filename="import_module.k",
            line=3,
            column=11,
            locations=[
                Location(
                    uri="pkg/parent.k",
                    range=Range(
                        start=Position(line=0, character=7),
                        end=Position(line=0, character=13),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="import_module.k",
            line=3,
            column=9,
            locations=[
                Location(
                    uri="import_module.k",
                    range=Range(
                        start=Position(line=0, character=22),
                        end=Position(line=0, character=23),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="import_module.k",
            line=1,
            column=23,
            locations=[
                Location(
                    uri="import_module.k",
                    range=Range(
                        start=Position(line=0, character=22),
                        end=Position(line=0, character=23),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="import_module.k",
            line=7,
            column=9,
            locations=[],
        ),
        GoToDefTestCase(
            filename="attr.k",
            line=9,
            column=10,
            locations=[
                Location(
                    uri="attr.k",
                    range=Range(
                        start=Position(line=3, character=4),
                        end=Position(line=3, character=9),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="attr.k",
            line=14,
            column=17,
            locations=[
                Location(
                    uri="attr.k",
                    range=Range(
                        start=Position(line=0, character=0),
                        end=Position(line=0, character=16),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="inherit.k",
            line=1,
            column=8,
            locations=[
                Location(
                    uri="inherit.k",
                    range=Range(
                        start=Position(line=0, character=7),
                        end=Position(line=0, character=13),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="inherit.k",
            line=9,
            column=27,
            locations=[
                Location(
                    uri="inherit.k",
                    range=Range(
                        start=Position(line=1, character=4),
                        end=Position(line=1, character=8),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="inherit.k",
            line=10,
            column=14,
            locations=[],
        ),
        GoToDefTestCase(
            filename="schema.k",
            line=5,
            column=9,
            locations=[
                Location(
                    uri="schema.k",
                    range=Range(
                        start=Position(line=0, character=7),
                        end=Position(line=0, character=13),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="schema_index_signature.k",
            line=5,
            column=9,
            locations=[
                Location(
                    uri="schema_index_signature.k",
                    range=Range(
                        start=Position(line=1, character=5),
                        end=Position(line=1, character=9),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="parent_attr.k",
            line=7,
            column=5,
            locations=[
                Location(
                    uri="pkg/parent.k",
                    range=Range(
                        start=Position(line=1, character=4),
                        end=Position(line=1, character=8),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="list_comp.k",
            line=6,
            column=17,
            locations=[
                Location(
                    uri="list_comp.k",
                    range=Range(
                        start=Position(line=5, character=22),
                        end=Position(line=5, character=23),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="list_comp.k",
            line=7,
            column=14,
            # name="e",
            locations=[
                Location(
                    uri="list_comp.k",
                    range=Range(
                        start=Position(line=0, character=4),
                        end=Position(line=0, character=5),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="inherit.k",
            line=6,
            column=21,
            locations=[
                Location(
                    uri="inherit.k",
                    range=Range(
                        start=Position(line=1, character=4),
                        end=Position(line=1, character=8),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="inherit_pkg.k",
            line=4,
            column=21,
            locations=[
                Location(
                    uri=str(pathlib.Path("pkg") / "parent.k"),
                    range=Range(
                        start=Position(line=1, character=4),
                        end=Position(line=1, character=8),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="inherit_pkg.k",
            line=3,
            column=12,
            locations=[
                Location(
                    uri=str("inherit_pkg.k"),
                    range=Range(
                        start=Position(line=0, character=7),
                        end=Position(line=0, character=10),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="inherit_pkg.k",
            line=7,
            column=1,
            locations=[],
        ),
        GoToDefTestCase(
            filename="invalid_semantic.k",
            line=2,
            column=5,
            locations=[
                Location(
                    uri="invalid_semantic.k",
                    range=Range(
                        start=Position(line=0, character=0),
                        end=Position(line=0, character=1),
                    ),
                )
            ],
        ),
        GoToDefTestCase(
            filename="invalid_grammar.k",
            line=1,
            column=1,
            locations=[],
        ),
    ]

    def test_go_to_def(self):
        for i, t_case in enumerate(self.test_cases):
            got_locations = go_to_def.go_to_def(
                pos=ast.Position(
                    filename=str(_DIR_PATH / t_case.filename),
                    line=t_case.line,
                    column=t_case.column,
                ),
            )
            expect_locations = [
                Location(uri=str(_DIR_PATH / loc.uri), range=loc.range)
                for loc in t_case.locations
            ]
            self.assertEqual(
                expect_locations,
                got_locations,
                f"err go to def for case[{i}], from pos:{t_case.filename}:{t_case.line}:{t_case.column}\nexpect: {expect_locations}, got: {got_locations}",
            )


class NoneInputTest(unittest.TestCase):
    def test_find_declaration_None_input(self):
        self.assertIsNone(go_to_def.find_declaration(None, None, None))

    def test_find_declaration_by_scope_obj_None_input(self):
        self.assertIsNone(
            go_to_def.find_declaration_by_scope_obj(None, None, None, None)
        )

    def test_find_declaration_obj_by_pos_and_name_None_input(self):
        self.assertIsNone(
            go_to_def.find_declaration_obj_by_pos_and_name(None, None, None)
        )

    def test_find_inner_name_None_input(self):
        self.assertIsNone(go_to_def.find_inner_name(None, None, None))

    def test_find_attr_by_name_None_input(self):
        self.assertIsNone(go_to_def.find_attr_by_name(None, None, None))


def test_find_declaration_obj_by_pos_and_name():
    pos: ast.Position = ast.Position(filename="simple.k", line=2, column=5)
    _, prog_scope = common.file_or_prog_to_scope(file_path=_DIR_PATH / "simple.k")
    assert (
        go_to_def.find_declaration_obj_by_pos_and_name(
            pos=pos, name="d", prog_scope=prog_scope
        )
        is None
    )


if __name__ == "__main__":
    unittest.main(verbosity=2)
