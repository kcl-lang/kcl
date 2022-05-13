# Copyright 2020 The KCL Authors. All rights reserved.

import io
import unittest
import pathlib

import kclvm.encoding.protobuf.parser as parser
import kclvm.encoding.protobuf.printer as printer


simple_test_case = """\
syntax = "proto3";
package kclvm.runtime.api;
import public "other.proto";
option java_package = "com.example.foo";

enum EnumAllowingAlias {
    option allow_alias = true;
    UNKNOWN = 0;
    STARTED = 1;
    RUNNING = 2 [(custom_option) = "hello world"];
}

message outer {
    option my_option.a = true;
    option my_option.b = false;

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

message Foo {
    reserved 2, 15, 9 to 11;
    reserved "foo", "bar";
}

message Args {
    bool result = 1;
}

service Service {
    rpc Method(Args) returns (Args);
}
"""
simple_test_case_expected = """\
syntax = "proto3";
package kclvm.runtime.api;
import public "other.proto";
option java_package = "com.example.foo";

enum EnumAllowingAlias {
    option allow_alias = true;
    UNKNOWN = 0;
    STARTED = 1;
    RUNNING = 2 [(custom_option) = "hello world"];
}

message outer {
    option my_option.a = true;
    option my_option.b = false;

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

message Foo {
    reserved 2, 15, 9 to 11;
    reserved "foo", "bar";
}

message Args {
    bool result = 1;
}

service Service {
    rpc Method(Args) returns (Args);
}
"""


class TestProtobufPrinter(unittest.TestCase):
    def setUp(self):
        super().setUp()
        self.maxDiff = None

    def test_protobuf_printer(self):
        proto = parser.parse_code(simple_test_case)
        proto_text = printer.print_node_to_string(proto)
        self.assertEqual(proto_text, simple_test_case_expected)

    def test_interleave(self):
        out = io.StringIO()
        printer.BasePrinter.interleave(
            lambda: out.write(", "), lambda n: out.write(str(n)), []
        )
        printer.BasePrinter.interleave(
            lambda: out.write(", "), lambda n: out.write(str(n)), [1, 2, 3]
        )
        self.assertEqual(out.getvalue(), "1, 2, 3")


if __name__ == "__main__":
    unittest.main(verbosity=2)
