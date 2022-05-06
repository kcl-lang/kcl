import unittest
from kclvm.compiler.build.utils import units


class UnitsTest(unittest.TestCase):

    def test_cal_num_si(self) -> None:
        """
        Test cal_num function with SI suffix
        """
        cases = [
            {"value": 1, "suffix": "n", "expected": 1e-09},
            {"value": 1, "suffix": "u", "expected": 1e-06},
            {"value": 1, "suffix": "m", "expected": 0.001},
            {"value": 1, "suffix": "", "expected": 1},
            {"value": 1, "suffix": "k", "expected": 1_000},
            {"value": 1, "suffix": "K", "expected": 1_000},
            {"value": 1, "suffix": "M", "expected": 1_000_000},
            {"value": 1, "suffix": "G", "expected": 1_000_000_000},
            {"value": 1, "suffix": "T", "expected": 1_000_000_000_000},
            {"value": 1, "suffix": "P", "expected": 1_000_000_000_000_000},
        ]

        for case in cases:
            real = units.cal_num(case["value"], case["suffix"])
            self.assertEqual(real, case["expected"])

    def test_cal_num_iec(self) -> None:
        """
        Test cal_num function with IEC suffix
        """
        cases = [
            {"value": 1, "suffix": "Ki", "expected": 1024},
            {"value": 1, "suffix": "Mi", "expected": 1024 ** 2},
            {"value": 1, "suffix": "Gi", "expected": 1024 ** 3},
            {"value": 1, "suffix": "Ti", "expected": 1024 ** 4},
            {"value": 1, "suffix": "Pi", "expected": 1024 ** 5},
        ]

        for case in cases:
            real = units.cal_num(case["value"], case["suffix"])
            self.assertEqual(real, case["expected"])

    def test_cal_num_invalid_suffix(self) -> None:
        """
        Test cal_num function with invalid suffix
        """
        cases = [
            {"value": 1, "suffix": "x"},
            {"value": 1, "suffix": "ki"},
            {"value": 1, "suffix": "mi"},
            {"value": 1, "suffix": "ui"},
            {"value": 1, "suffix": "ni"},
        ]

        for case in cases:
            exception = None

            try:
                units.cal_num(case["value"], case["suffix"])
            except ValueError as e:
                exception = e
                self.assertEqual(str(e), f"Invalid suffix { case['suffix'] }")

            if not exception:
                self.assertFalse(True, f"ValueError should be thrown, case: {case['suffix']}")

    def test_to_quantity_si(self) -> None:
        """
        Test to_quantity function with SI suffix
        """
        cases = [
            {"quantity": "1n", "expected": 1e-09},
            {"quantity": "1u", "expected": 1e-06},
            {"quantity": "1m", "expected": 0.001},
            {"quantity": "1", "expected": 1},
            {"quantity": "1k", "expected": 1_000},
            {"quantity": "1K", "expected": 1_000},
            {"quantity": "1M", "expected": 1_000_000},
            {"quantity": "1G", "expected": 1_000_000_000},
            {"quantity": "1T", "expected": 1_000_000_000_000},
            {"quantity": "1P", "expected": 1_000_000_000_000_000},
        ]

        for case in cases:
            real = units.to_quantity(case["quantity"])
            self.assertEqual(real, case["expected"])

    def test_to_quantity_iec(self) -> None:
        """
        Test to_quantity function with SI suffix
        """
        cases = [
            {"quantity": "1Ki", "expected": 1024},
            {"quantity": "1Mi", "expected": 1024 ** 2},
            {"quantity": "1Gi", "expected": 1024 ** 3},
            {"quantity": "1Ti", "expected": 1024 ** 4},
            {"quantity": "1Pi", "expected": 1024 ** 5},
        ]

        for case in cases:
            real = units.to_quantity(case["quantity"])
            self.assertEqual(real, case["expected"])

    def test_to_quantity_invalid_suffix(self) -> None:
        """
        Test to_quantity function with invalid quantity
        """
        cases = [
            {"quantity": "1x"},
            {"quantity": "1ki"},
            {"quantity": "1mi"},
            {"quantity": "1ui"},
            {"quantity": "1ni"},
            {"quantity": "1Kii"},
            {"quantity": "x"},
        ]

        for case in cases:
            try:
                units.to_quantity(case["quantity"])
            except ValueError as e:
                self.assertEqual(str(e), "invalid literal for int() with base 10: '{}'".format(case["quantity"]))

        cases = [
            {"quantity": ""},
            {"quantity": "ki"},
            {"quantity": "mi"},
            {"quantity": "ui"},
            {"quantity": "ni"},
        ]

        for case in cases:
            try:
                units.to_quantity(case["quantity"])
            except ValueError as e:
                self.assertEqual(str(e), "Number can't be empty")


if __name__ == "__main__":
    unittest.main(verbosity=2)
