# Copyright 2021 The KCL Authors. All rights reserved.
import unittest
import pathlib
from typing import Optional

import kclvm.tools.langserver.common as common
import kclvm.kcl.ast as ast
import kclvm.kcl.types.scope as scope

_TEST_PATH_NAME = "test_data"
_GO_TO_DEF_PATH_NAME = "go_to_def"
_DIR_PATH = (
    pathlib.Path(__file__)
    .parent.joinpath(_TEST_PATH_NAME)
    .joinpath(_GO_TO_DEF_PATH_NAME)
)


class PosToScopeTest(unittest.TestCase):
    def test_pos_to_scope(self):
        # check invalid pos
        pos = ast.Position(line=0, column=0)
        _, got, _ = common.pos_to_scope(pos)
        self.assertIsNone(
            got, f"find scope from invalid position {pos}, expect: None, got: {got}"
        )

        file_path = "invalid_path"
        _, got, _ = common.pos_to_scope(
            pos=ast.Position(filename=file_path, line=1, column=1)
        )
        self.assertIsNone(
            got,
            f"find scope from invalid file path {file_path}, expect: None, got: {got}",
        )

        file_path = str(_DIR_PATH / "schema.k")
        prog = common.file_to_prog(file_path)
        prog.pkgs['__main__'][0].body[0].body[0].type_str = None

        _, got = common.file_or_prog_to_scope(prog, file_path)
        self.assertIsInstance(
            got,
            scope.ProgramScope,
            f"find scope from invalid prog {file_path}, expect: None, got: {got}"
        )


class PosToNodeTestCase:
    def __init__(
        self,
        filename: str,
        line: int,
        column: int,
        name: Optional[str] = None,
        start_pos: Optional[ast.Position] = None,
        end_pos: Optional[ast.Position] = None,
    ):
        self.filename: str = filename
        self.line: int = line
        self.column: int = column
        self.name: Optional[str] = name
        self.start_pos: Optional[ast.Position] = start_pos
        self.end_pos: Optional[ast.Position] = end_pos


class PosToNodeTest(unittest.TestCase):
    _cases = [
        PosToNodeTestCase(
            filename="member_access.k",
            line=6,
            column=1,
            name="name",
            start_pos=ast.Position(line=6, column=1),
            end_pos=ast.Position(line=6, column=5),
        ),
        PosToNodeTestCase(
            filename="schema.k",
            line=5,
            column=9,
            name="Person",
            start_pos=ast.Position(line=5, column=9),
            end_pos=ast.Position(line=5, column=15),
        ),
        PosToNodeTestCase(
            filename="simple.k",
            line=2,
            column=5,
            name="d",
            start_pos=ast.Position(line=2, column=5),
            end_pos=ast.Position(line=2, column=6),
        ),
        PosToNodeTestCase(
            filename="invalid_grammar.k",
            line=1,
            column=1,
        ),
        PosToNodeTestCase(
            filename="",
            line=0,
            column=0,
        ),
        PosToNodeTestCase(
            filename="invalid_path.k",
            line=1,
            column=1,
        ),
        PosToNodeTestCase(
            filename="simple.k",
            line=1,
            column=3,
        ),
    ]

    def test_pos_to_node(self):
        for t_case in self._cases:
            _, node = common.pos_to_node(
                pos=ast.Position(
                    filename=str(_DIR_PATH / t_case.filename),
                    line=t_case.line,
                    column=t_case.column,
                ),
            )
            if node:
                self.assertEqual(
                    str(_DIR_PATH / t_case.filename),
                    node.filename,
                    msg="filename not match",
                )
                self.assertEqual(
                    t_case.start_pos.line,
                    node.line,
                    msg=f"start line not match. filename={t_case.filename}",
                )
                self.assertEqual(
                    t_case.start_pos.column,
                    node.column,
                    msg=f"start column not match. filename={t_case.filename}",
                )
                self.assertEqual(
                    t_case.end_pos.line,
                    node.end_line,
                    msg=f"end line not match. filename={t_case.filename}",
                )
                self.assertEqual(
                    t_case.end_pos.column,
                    node.end_column,
                    msg=f"end column not match. filename={t_case.filename}",
                )
                if isinstance(node, ast.Name):
                    self.assertEqual(
                        t_case.name,
                        node.value,
                        msg=f"node value not match. filename={t_case.filename}",
                    )
            else:
                self.assertIsNone(
                    t_case.name,
                    msg=f"node value not match, filename: {t_case.filename}",
                )
                self.assertIsNone(
                    t_case.start_pos,
                    msg=f"node start pos not match, filename: {t_case.filename}",
                )
                self.assertIsNone(
                    t_case.end_pos,
                    msg=f"node end pos not match, filename: {t_case.filename}",
                )


class ScopeObjToLocationTest(unittest.TestCase):
    def test_scope_obj_to_location_None_input(self):
        self.assertIsNone(common.scope_obj_to_location(None))
        self.assertIsNone(common.scope_obj_to_location(scope.ScopeObject(name="mock",node=None,type=None)))

    def test_file_to_location_invalid_path(self):
        self.assertIsNone(common.file_to_location(filepath=""))
        self.assertIsNone(common.file_to_location(filepath="mock.txt"))


if __name__ == "__main__":
    unittest.main(verbosity=2)
