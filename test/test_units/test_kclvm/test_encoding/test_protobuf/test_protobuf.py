# Copyright 2020 The KCL Authors. All rights reserved.

import unittest
import pathlib

import kclvm.encoding.protobuf.protobuf as protobuf


simple_to_kcl_test_case = """\
syntax = "proto3";
import public "other.proto";
option java_package = "com.example.foo";

enum EnumAllowingAlias {
    option allow_alias = true;
    UNKNOWN = 0;
    STARTED = 1;
    RUNNING = 2 [(custom_option) = "hello world"];
}

message outer {
    option (my_option).a = true;
    message inner {
        int64 ival = 1;
    }
    repeated inner inner_message = 2;
    EnumAllowingAlias enum_field = 3;
    map<int32, string> my_map = 4;
    oneof x {
        int32 id = 1;
        string name = 2;
    }
}
"""
simple_to_kcl_test_case_expected = """\
type EnumAllowingAlias = 0 | 1 | 2
schema outer:
    inner_message: [inner]
    enum_field: EnumAllowingAlias
    my_map: {int:str}
    x: int | str

schema inner:
    ival: int

"""
simple_to_pb_test_case = """\
schema Inner:
    ival: int

schema Outer:
    inner: Inner
    inner_message: [Inner]
    my_map: {str:str}
    x: int | str
"""
simple_to_pb_test_case_expected = """\
syntax = "proto3";

message Inner {
    int64 ival = 1;
}

message Outer {
    Inner inner = 1;
    repeated Inner inner_message = 2;
    map<string, string> my_map = 3;
    oneof x {
        int x1 = 1;
        str x2 = 2;
    }
}
"""


class TestProtobufToKCLCode(unittest.TestCase):
    def test_to_kcl_simple_case(self):
        kcl_code = protobuf.protobuf_to_kcl(simple_to_kcl_test_case)
        self.assertEqual(kcl_code, simple_to_kcl_test_case_expected)

    def test_invalid_function_arguments(self):
        with self.assertRaises(ValueError):
            protobuf.convert_enum_to_type_alias(None)
        with self.assertRaises(ValueError):
            protobuf.convert_message_to_schema_list(None)
        with self.assertRaises(ValueError):
            protobuf.convert_schema_to_message(None)
        with self.assertRaises(ValueError):
            protobuf.convert_schema_attr_to_message_body(None)
        with self.assertRaises(ValueError):
            protobuf.convert_kcl_type_to_proto_type("")


class TestKCLToProtobufCode(unittest.TestCase):
    def test_to_pb_simple_case(self):
        kcl_code = protobuf.kcl_to_protobuf(simple_to_pb_test_case)
        self.assertEqual(kcl_code, simple_to_pb_test_case_expected)


if __name__ == "__main__":
    unittest.main(verbosity=2)
