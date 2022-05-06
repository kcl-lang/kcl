import json
import unittest
import pathlib

import kclvm.kcl.ast as ast
import kclvm.api.object as objpkg
import kclvm.tools.list_attribute.schema as schema
import kclvm.internal.gpyrpc.gpyrpc_pb2 as pb2


import google.protobuf.json_format as json_format


class KCLListAttrSchemaTest(unittest.TestCase):
    def test_get_schema_type_from_code(self):
        self.maxDiff = None
        testdata_path = pathlib.Path(__file__).parent.joinpath("schema_testdata")
        k_files = list(sorted(testdata_path.glob("*.k")))
        json_files = list(sorted(testdata_path.glob("*.k.json")))
        for k_file, json_file in zip(k_files, json_files):
            k_code = k_file.read_text()
            type_list = schema.get_schema_type_from_code("", code=k_code)
            type_list = [json_format.MessageToDict(t) for t in type_list]
            json_str = json_file.read_text()
            expected_data = json.loads(json_str)
            self.assertEqual(type_list, expected_data)

    def test_get_schema_type_from_code_with_schema_name_para(self):
        testdata_path = pathlib.Path(__file__).parent.joinpath("schema_testdata").joinpath("complex.k")
        k_code = testdata_path.read_text()
        schema_names = ["User", "HomeTown", "Custom", "Color"]
        for name in schema_names:
            type_list = schema.get_schema_type_from_code("", code=k_code, schema_name=name)
            self.assertEqual(len(type_list), 1)
        err_name = "ErrSchema"
        type_list = schema.get_schema_type_from_code("", code=k_code, schema_name=err_name)
        self.assertEqual(len(type_list), 0)

    def test_get_schema_type_from_code_invalid(self):
        with self.assertRaises(Exception):
            schema.get_schema_type_from_code(None, None)

    def test_kcl_type_obj_to_pb_kcl_type(self):
        self.assertEqual(schema.kcl_type_obj_to_pb_kcl_type(None), None)


if __name__ == "__main__":
    unittest.main(verbosity=2)
