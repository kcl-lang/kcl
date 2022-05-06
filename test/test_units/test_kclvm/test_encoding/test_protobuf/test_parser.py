# Copyright 2020 The KCL Authors. All rights reserved.

import unittest
import pathlib

import kclvm.encoding.protobuf as protobuf


simple_test_case = """\
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
    option (my_option).b = -1;
    option (my_option).c = "\\xAA";

    message inner {
        int64 ival = 1;
    }
    repeated inner inner_message = 2;
    EnumAllowingAlias enum_field = 3;
    map<int32, string> my_map = 4;
}
"""


class TestProtobufParseCode(unittest.TestCase):
    def test_simple_case(self):
        proto = protobuf.parse_code(simple_test_case)
        self.assertEqual(len(proto.statements), 4)
        self.assertEqual(proto.syntax, "proto3")
        self.assertIsInstance(proto.statements[0], protobuf.Import)
        self.assertIsInstance(proto.statements[1], protobuf.Option)
        self.assertIsInstance(proto.statements[2], protobuf.Enum)
        self.assertIsInstance(proto.statements[3], protobuf.Message)


if __name__ == "__main__":
    unittest.main(verbosity=2)
