# Copyright 2021 The KCL Authors. All rights reserved.

import pathlib
import unittest

import kclvm.kcl.error as kcl_error
import kclvm.api.object as objpkg
import kclvm.vm.code as vm_code
import kclvm.vm.runtime.evaluator.eval as eval


class EvaluatorTest(unittest.TestCase):
    def setUp(self):
        self._evaluator = eval.Evaluator()
        return super().setUp()

    def test_eval_binary_op(self):
        cases = [
            {
                "left": objpkg.KCLIntObject(1),
                "right": objpkg.KCLIntObject(1),
                "code": vm_code.Opcode.BINARY_ADD,
                "expected": objpkg.KCLIntObject(2),
            },
            {
                "left": objpkg.KCLIntObject(1),
                "right": objpkg.KCLIntObject(1),
                "code": vm_code.Opcode.BINARY_SUBTRACT,
                "expected": objpkg.KCLIntObject(0),
            },
        ]
        for case in cases:
            left, right, code, expected = (
                case["left"],
                case["right"],
                case["code"],
                case["expected"],
            )
            result = self._evaluator.eval_binary_op(left, right, code)
            self.assertEqual(result, expected)

    def test_eval_binary_op_invalid(self):
        cases = [
            {"left": None, "right": None, "code": vm_code.Opcode.BINARY_ADD},
            {
                "left": objpkg.KCLIntObject(1),
                "right": objpkg.KCLIntObject(1),
                "code": vm_code.Opcode.NOP,
            },
        ]
        for case in cases:
            left, right, code = case["left"], case["right"], case["code"]
            with self.assertRaises(Exception):
                self._evaluator.eval_binary_op(left, right, code)

    def test_eval_compare_op(self):
        cases = [
            {
                "left": objpkg.KCLIntObject(1),
                "right": objpkg.KCLIntObject(1),
                "code": vm_code.Opcode.COMPARE_EQUAL_TO,
                "expected": objpkg.TRUE_INSTANCE,
            },
            {
                "left": objpkg.KCLIntObject(1),
                "right": objpkg.KCLIntObject(1),
                "code": vm_code.Opcode.COMPARE_GREATER_THAN_OR_EQUAL_TO,
                "expected": objpkg.TRUE_INSTANCE,
            },
        ]
        for case in cases:
            left, right, code, expected = (
                case["left"],
                case["right"],
                case["code"],
                case["expected"],
            )
            result = self._evaluator.eval_compare_op(left, right, code)
            self.assertEqual(result, expected)

    def test_eval_compare_op_invalid(self):
        cases = [
            {"left": None, "right": None, "code": vm_code.Opcode.BINARY_ADD},
            {
                "left": objpkg.KCLIntObject(1),
                "right": objpkg.KCLIntObject(1),
                "code": vm_code.Opcode.NOP,
            },
        ]
        for case in cases:
            left, right, code = case["left"], case["right"], case["code"]
            with self.assertRaises(Exception):
                self._evaluator.eval_compare_op(left, right, code)

    def test_eval_unary_op(self):
        cases = [
            {
                "operand": objpkg.KCLIntObject(1),
                "code": vm_code.Opcode.UNARY_POSITIVE,
                "expected": objpkg.KCLIntObject(1),
            },
            {
                "operand": objpkg.KCLIntObject(1),
                "code": vm_code.Opcode.UNARY_NEGATIVE,
                "expected": objpkg.KCLIntObject(-1),
            },
        ]
        for case in cases:
            operand, code, expected = (
                case["operand"],
                case["code"],
                case["expected"],
            )
            result = self._evaluator.eval_unary_op(operand, code)
            self.assertEqual(result, expected)

    def test_eval_unary_op_invalid(self):
        cases = [
            {"operand": None, "code": vm_code.Opcode.BINARY_ADD},
            {
                "operand": objpkg.KCLIntObject(1),
                "code": vm_code.Opcode.NOP,
            },
        ]
        for case in cases:
            operand, code = case["operand"], case["code"]
            with self.assertRaises(Exception):
                self._evaluator.eval_unary_op(operand, code)

    def test_set_item(self):
        cases = [
            {
                "operand": objpkg.to_kcl_obj({"key": "value"}),
                "item": objpkg.KCLStringObject("key"),
                "value": objpkg.KCLStringObject("override"),
                "expected": objpkg.to_kcl_obj({"key": "override"}),
            },
            {
                "operand": objpkg.to_kcl_obj([0, 0]),
                "item": objpkg.KCLIntObject(0),
                "value": objpkg.KCLIntObject(1),
                "expected": objpkg.to_kcl_obj([1, 0]),
            },
        ]
        for case in cases:
            operand, item, value, expected = case["operand"], case["item"], case["value"], case["expected"]
            result = self._evaluator.set_item(operand, item, value)
            self.assertEqual(result, expected)

    def test_format_value(self):
        cases = [
            {
                "operand": objpkg.to_kcl_obj({"key": "value"}),
                "format_spec": None,
                "expected": objpkg.KCLStringObject(value="{'key': 'value'}"),
            },
            {
                "operand": objpkg.to_kcl_obj({"key": "value"}),
                "format_spec": "#json",
                "expected": objpkg.KCLStringObject(value='{"key": "value"}'),
            },
            {
                "operand": objpkg.to_kcl_obj({"key": "value"}),
                "format_spec": "#yaml",
                "expected": objpkg.KCLStringObject(value='key: value\n'),
            },
        ]
        for case in cases:
            operand, format_spec, expected = case["operand"], case["format_spec"], case["expected"]
            result = self._evaluator.format_value(operand, format_spec)
            self.assertEqual(result, expected)


if __name__ == "__main__":
    unittest.main(verbosity=2)
