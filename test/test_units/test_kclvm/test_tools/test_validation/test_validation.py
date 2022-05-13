#! /usr/bin/env python3

import json
import unittest
import pathlib

from kclvm.kcl.error import KCLException
from kclvm.tools.validation import validate_code, validate_code_with_attr_data


class KCLValidationTest(unittest.TestCase):
    def test_validate_code_normal_json_data(self):
        case_path = pathlib.Path(__file__).parent.joinpath("json_test_data")
        json_data_list = sorted(case_path.glob("*.k.json"))
        code_string_list = sorted(case_path.glob("*.k"))
        for json_data_file, code_string_file in zip(json_data_list, code_string_list):
            json_data = pathlib.Path(json_data_file).read_text()
            code_string = pathlib.Path(code_string_file).read_text()
            self.assertTrue(validate_code(json_data, code_string))

    def test_validate_code_invalid_json_data(self):
        case_path = pathlib.Path(__file__).parent.joinpath("json_invalid_test_data")
        json_data_list = sorted(case_path.glob("*.k.json"))
        code_string_list = sorted(case_path.glob("*.k"))
        for json_data_file, code_string_file in zip(json_data_list, code_string_list):
            json_data = pathlib.Path(json_data_file).read_text()
            code_string = pathlib.Path(code_string_file).read_text()
            with self.assertRaises(KCLException):
                validate_code(json_data, code_string)

    def test_validate_code_invalid_argument(self):
        invalid_argument_cases = [
            {"data": None, "code": None, "format": "json", "attribute_name": "value"},
            {"data": "1", "code": None, "format": "json", "attribute_name": "value"},
            {
                "data": None,
                "code": "a = 1",
                "format": "json",
                "attribute_name": "value",
            },
            {
                "data": "1",
                "code": "assert value >= 1",
                "format": None,
                "attribute_name": "value",
            },
            {
                "data": "1",
                "code": "assert value >= 1",
                "format": "err_format",
                "attribute_name": "value",
            },
            {
                "data": "1",
                "code": "assert value >= 1",
                "format": "json",
                "attribute_name": None,
            },
        ]
        for case in invalid_argument_cases:
            with self.assertRaises(ValueError, msg=f"{case}"):
                data, code, format, attribute_name = (
                    case["data"],
                    case["code"],
                    case["format"],
                    case["attribute_name"],
                )
                validate_code(data, code, format=format, attribute_name=attribute_name)

    def test_validate_code_with_attr_data_normal_json_data(self):
        case_path = pathlib.Path(__file__).parent.joinpath("json_test_data")
        json_data_list = sorted(case_path.glob("*.k.json"))
        code_string_list = sorted(case_path.glob("*.k"))
        attr_name = "value"
        for json_data_file, code_string_file in zip(json_data_list, code_string_list):
            json_data = pathlib.Path(json_data_file).read_text()
            json_data = json.dumps({attr_name: json.loads(json_data)})
            code_string = pathlib.Path(code_string_file).read_text()
            self.assertTrue(validate_code_with_attr_data(json_data, code_string))

    def test_validate_code_with_attr_data_invalid_json_data(self):
        case_path = pathlib.Path(__file__).parent.joinpath("json_test_data")
        json_data_list = sorted(case_path.glob("*.k.json"))
        code_string_list = sorted(case_path.glob("*.k"))
        attr_name = "value"
        for json_data_file, code_string_file in zip(json_data_list, code_string_list):
            json_data = pathlib.Path(json_data_file).read_text()
            json_data = json.dumps({attr_name: json.loads(json_data), "err_key": {}})
            with self.assertRaises(ValueError):
                validate_code_with_attr_data(json_data, "")


if __name__ == "__main__":
    unittest.main(verbosity=2)
