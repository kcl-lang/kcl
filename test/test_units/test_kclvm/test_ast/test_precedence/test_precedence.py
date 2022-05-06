#! /usr/bin/env python3

import unittest

import kclvm.kcl.ast as ast

import kclvm.kcl.ast.lark_token as lark_token


class KCLPrecedenceTest(unittest.TestCase):
    """KCL AST operator precedence test"""

    def test_precedence(self):
        self.assertEqual(ast.precedence(None), 0)
        self.assertEqual(ast.precedence(ast.lark_token.LarkToken.L_simple_expr), 0)
        self.assertEqual(ast.precedence(ast.AugOp.Add), 0)
        self.assertEqual(ast.precedence(ast.AugOp.Sub), 0)
        self.assertEqual(ast.precedence(ast.AugOp.Mul), 0)
        self.assertEqual(ast.precedence(ast.AugOp.Div), 0)
        self.assertEqual(ast.precedence(ast.AugOp.Mod), 0)
        self.assertEqual(ast.precedence(ast.AugOp.Pow), 0)
        self.assertEqual(ast.precedence(ast.AugOp.LShift), 0)
        self.assertEqual(ast.precedence(ast.AugOp.RShift), 0)
        self.assertEqual(ast.precedence(ast.AugOp.BitOr), 0)
        self.assertEqual(ast.precedence(ast.AugOp.BitXor), 0)
        self.assertEqual(ast.precedence(ast.AugOp.BitAnd), 0)
        self.assertEqual(ast.precedence(ast.AugOp.FloorDiv), 0)
        self.assertEqual(ast.precedence(ast.BinOp.Add), 9)
        self.assertEqual(ast.precedence(ast.BinOp.Sub), 9)
        self.assertEqual(ast.precedence(ast.BinOp.Mul), 10)
        self.assertEqual(ast.precedence(ast.BinOp.Div), 10)
        self.assertEqual(ast.precedence(ast.BinOp.Mod), 10)
        self.assertEqual(ast.precedence(ast.BinOp.Pow), 12)
        self.assertEqual(ast.precedence(ast.BinOp.LShift), 8)
        self.assertEqual(ast.precedence(ast.BinOp.RShift), 8)
        self.assertEqual(ast.precedence(ast.BinOp.BitOr), 6)
        self.assertEqual(ast.precedence(ast.BinOp.BitXor), 5)
        self.assertEqual(ast.precedence(ast.BinOp.BitAnd), 7)
        self.assertEqual(ast.precedence(ast.BinOp.FloorDiv), 10)
        self.assertEqual(ast.precedence(ast.BinOp.And), 2)
        self.assertEqual(ast.precedence(ast.BinOp.Or), 1)
        self.assertEqual(ast.precedence(ast.CmpOp.Eq), 4)
        self.assertEqual(ast.precedence(ast.CmpOp.NotEq), 4)
        self.assertEqual(ast.precedence(ast.CmpOp.Lt), 4)
        self.assertEqual(ast.precedence(ast.CmpOp.LtE), 4)
        self.assertEqual(ast.precedence(ast.CmpOp.Gt), 4)
        self.assertEqual(ast.precedence(ast.CmpOp.GtE), 4)
        self.assertEqual(ast.precedence(ast.CmpOp.Is), 4)
        self.assertEqual(ast.precedence(ast.CmpOp.In), 4)
        self.assertEqual(ast.precedence(ast.CmpOp.Not), 4)
        self.assertEqual(ast.precedence(ast.CmpOp.IsNot), 4)
        self.assertEqual(ast.precedence(ast.CmpOp.NotIn), 4)
        self.assertEqual(ast.precedence(ast.UnaryOp.UAdd), 11)
        self.assertEqual(ast.precedence(ast.UnaryOp.USub), 11)
        self.assertEqual(ast.precedence(ast.UnaryOp.Invert), 11)
        self.assertEqual(ast.precedence(ast.UnaryOp.Not), 3)


if __name__ == "__main__":
    unittest.main(verbosity=2)
