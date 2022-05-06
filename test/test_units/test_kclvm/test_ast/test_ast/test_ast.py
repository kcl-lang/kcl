# Copyright 2020 The KCL Authors. All rights reserved.

import typing
import pathlib
import hashlib
import unittest

import kclvm.kcl.ast as ast
import kclvm.compiler.parser as parser


class TestAst(unittest.TestCase):
    INIT_LINE = 0
    INIT_COLUMN = 0
    INIT_END_LINE = 0
    INIT_END_COLUMN = 0
    ast_node = ast.AST(INIT_LINE, INIT_COLUMN, INIT_END_LINE, INIT_END_COLUMN)

    set_methods = [
        "set_line",
        "set_column",
        "set_end_line",
        "set_end_column",
    ]

    get_methods = [
        "get_line",
        "get_column",
        "get_end_line",
        "get_end_column",
    ]

    offset_method = [
        "offset_line",
        "offset_column",
        "offset_end_line",
        "offset_end_column",
    ]

    # line, column, end_line, end_column
    test_params = [0, 10, 22, 23]
    offset_parmas = [-1, 10, 0]

    def test_ast_offset(self):
        for i, method in enumerate(self.offset_method):
            test_method = getattr(self.ast_node, method)
            for offset_value in self.offset_parmas:
                get_method = getattr(self.ast_node, self.get_methods[i])
                before = get_method()
                test_method(offset_value)
                assert get_method() == before + offset_value

    def test_ast_set_get_line_column(self):
        for i, method in enumerate(self.set_methods):
            test_method = getattr(self.ast_node, method)
            test_method(self.test_params[i])

        for i, method in enumerate(self.get_methods):
            test_method = getattr(self.ast_node, method)
            assert test_method() == self.test_params[i]

    def test_set_invalid(self):
        for method in self.set_methods:
            test_method = getattr(self.ast_node, method)
            with self.assertRaises(AssertionError):
                test_method("-1")
            with self.assertRaises(AssertionError):
                self.ast_node.set_line(-1)

    def test_offset_line_invalid(self):
        for method in self.offset_method:
            test_method = getattr(self.ast_node, method)
            with self.assertRaises(AssertionError):
                test_method("-1")

    def test_position_less_than(self):
        _ONE = "one"
        _OTHER = "other"
        _RESULT = "result"
        test_cases = [
            {
                # position invalid
                _ONE: ast.Position(filename="one.k", line=0, column=1),
                _OTHER: ast.Position(filename="one.k", line=0, column=1),
                _RESULT: False,
            },
            {
                # different filename
                _ONE: ast.Position(filename="one.k", line=1, column=1),
                _OTHER: ast.Position(filename="other.k", line=1, column=1),
                _RESULT: False,
            },
            {
                # line number less than
                _ONE: ast.Position(filename="one.k", line=1, column=1),
                _OTHER: ast.Position(filename="one.k", line=2, column=1),
                _RESULT: True,
            },
            {
                # line number larger than
                _ONE: ast.Position(filename="one.k", line=2, column=1),
                _OTHER: ast.Position(filename="one.k", line=1, column=1),
                _RESULT: False,
            },
            {
                # line number equal, column number less than
                _ONE: ast.Position(filename="one.k", line=1, column=0),
                _OTHER: ast.Position(filename="one.k", line=1, column=1),
                _RESULT: True,
            },
        ]
        for t in test_cases:
            expect = t[_RESULT]
            got = t[_ONE].less(t[_OTHER])
            assert (
                expect == got
            ), f"position less than check between {t[_ONE]} and {t[_OTHER]}, expect: {expect}, got: {got}"

    def test_position_valid(self):
        _POS = "pos"
        _RESULT = "result"
        test_cases = [
            {
                # empty filename
                _POS: ast.Position(line=1, column=0),
                _RESULT: False,
            },
            {
                # line number < 1
                _POS: ast.Position(filename="pos.k", line=0, column=0),
                _RESULT: False,
            },
        ]
        for t in test_cases:
            expect = t[_RESULT]
            got = t[_POS].is_valid()
            assert (
                expect == got
            ), f"position valid on {t[_POS]}, expect: {expect}, got: {got}"

    def test_position_less_equal(self):
        _ONE = "one"
        _OTHER = "other"
        _RESULT = "result"
        test_cases = [
            {
                # position invalid
                _ONE: ast.Position(filename="one.k", line=0, column=1),
                _OTHER: ast.Position(filename="one.k", line=0, column=1),
                _RESULT: False,
            },
            {
                # different filename
                _ONE: ast.Position(filename="one.k", line=1, column=1),
                _OTHER: ast.Position(filename="other.k", line=1, column=1),
                _RESULT: False,
            },
            {
                # position less than
                _ONE: ast.Position(filename="one.k", line=1, column=1),
                _OTHER: ast.Position(filename="one.k", line=2, column=1),
                _RESULT: True,
            },
            {
                # position equal
                _ONE: ast.Position(filename="one.k", line=1, column=1),
                _OTHER: ast.Position(filename="one.k", line=1, column=1),
                _RESULT: True,
            },
        ]

        for t in test_cases:
            expect = t[_RESULT]
            got = t[_ONE].less_equal(t[_OTHER])
            assert (
                expect == got
            ), f"position less equal check between {t[_ONE]} and {t[_OTHER]}, expect: {expect}, got: {got}"

    def test_position_equal(self):
        _ONE = "one"
        _OTHER = "other"
        _RESULT = "result"
        test_cases = [
            {
                # position equal
                _ONE: ast.Position(filename="one.k", line=0, column=1),
                _OTHER: ast.Position(filename="one.k", line=0, column=1),
                _RESULT: True,
            },
            {
                # position not equal
                _ONE: ast.Position(filename="one.k", line=0, column=1),
                _OTHER: ast.Position(filename="one.k", line=0, column=2),
                _RESULT: False,
            },
        ]
        for t in test_cases:
            expect = t[_RESULT]
            got = t[_ONE] == (t[_OTHER])
            assert (
                expect == got
            ), f"position equal check between {t[_ONE]} and {t[_OTHER]}, expect: {expect}, got: {got}"

    def test_get_check_sum(self):
        filename = str(pathlib.Path(__file__).parent.joinpath("test_data/check_sum.k"))
        prog = parser.LoadProgram(filename)
        with open(filename, "rb") as f:
            check_sum_expected = hashlib.md5()
            check_sum_expected.update(filename.encode("utf-8"))
            check_sum_expected.update(f.read())
            self.assertEqual(prog.get_check_sum(), check_sum_expected.hexdigest())

    def test_GetArgDefault_invalid(self):
        arg = ast.Arguments()
        self.assertEqual(arg.GetArgDefault(10), None)

    def test_find_nearest_parent_by_type(self):
        prog = parser.LoadProgram("mock.k", k_code_list=["a=1"], set_ast_parent=True)
        target_identifier = prog.pkgs[prog.MAIN_PKGPATH][0].body[0].targets[0]
        self.assertIsNotNone(target_identifier)
        self.assertIsNotNone(target_identifier.parent)
        nearest_schema_expr = target_identifier.find_nearest_parent_by_type(tpe=ast.SchemaExpr)
        self.assertIsNone(nearest_schema_expr)
        

if __name__ == "__main__":
    unittest.main(verbosity=2)
