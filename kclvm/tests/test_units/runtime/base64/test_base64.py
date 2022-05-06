# Copyright 2021 The KCL Authors. All rights reserved.

import typing
import unittest

import kclvm_runtime

# https://github.com/python/cpython/blob/main/Lib/test/test_base64.py

# encode(value: str, encoding: str = "utf-8") -> str
# decode(value: str, encoding: str = "utf-8") -> str

# kclvm_base64_encode
# kclvm_base64_decode

_Dylib = kclvm_runtime.KclvmRuntimeDylib()


class kclx_Base64:
    def __init__(self, dylib_=None):
        self.dylib = dylib_ if dylib_ else _Dylib

    def encodebytes(self, value: bytes) -> str:
        return self.encode(str(value, "utf-8"))

    def decodebytes(self, value: bytes) -> str:
        return self.decode(str(value, "utf-8"))

    def encode(self, value: str) -> str:
        return self.dylib.Invoke(f"base64.encode", value) + "\n"

    def decode(self, value: str) -> str:
        return self.dylib.Invoke(f"base64.decode", value)


base64 = kclx_Base64(_Dylib)


class BaseTest(unittest.TestCase):
    def _strip(delf, s: str) -> str:
        s = s.replace("\t", "")
        s = s.replace("\n", "")
        s = s.replace(" ", "")
        s = s.strip()

    def checkequal(self, expect, got):
        if isinstance(expect, (bytes, bytearray)):
            expect = str(expect, "utf8")
        if isinstance(got, (bytes, bytearray)):
            got = str(got, "utf8")

        expect = expect.strip()
        got = got.strip()

        expect = self._strip(expect)
        got = self._strip(got)

        self.assertEqual(expect, got)

    def test_encodebytes(self):
        eq = self.checkequal
        eq(base64.encodebytes(b"www.python.org"), b"d3d3LnB5dGhvbi5vcmc=\n")
        eq(base64.encodebytes(b"a"), b"YQ==\n")
        eq(base64.encodebytes(b"ab"), b"YWI=\n")
        eq(base64.encodebytes(b"abc"), b"YWJj\n")
        eq(base64.encodebytes(b""), b"")
        eq(
            base64.encodebytes(
                b"abcdefghijklmnopqrstuvwxyz"
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"
                b"0123456789!@#0^&*();:<>,. []{}"
            ),
            b"YWJjZGVmZ2hpamtsbW5vcHFyc3R1dnd4eXpBQkNE"
            b"RUZHSElKS0xNTk9QUVJTVFVWV1hZWjAxMjM0\nNT"
            b"Y3ODkhQCMwXiYqKCk7Ojw+LC4gW117fQ==\n",
        )
        # Non-bytes
        eq(base64.encodebytes(bytearray(b"abc")), b"YWJj\n")
        eq(base64.encodebytes(memoryview(b"abc")), b"YWJj\n")
        # eq(base64.encodebytes(array("B", b"abc")), b"YWJj\n")

    def test_decodebytes(self):
        eq = self.checkequal
        eq(base64.decodebytes(b"d3d3LnB5dGhvbi5vcmc="), b"www.python.org")
        eq(base64.decodebytes(b"YQ=="), b"a")
        eq(base64.decodebytes(b"YWI="), b"ab")
        eq(base64.decodebytes(b"YWJj"), b"abc")
        eq(
            base64.decodebytes(
                b"YWJjZGVmZ2hpamtsbW5vcHFyc3R1dnd4eXpBQkNE"
                b"RUZHSElKS0xNTk9QUVJTVFVWV1hZWjAxMjM0NT"
                b"Y3ODkhQCMwXiYqKCk7Ojw+LC4gW117fQ=="
            ),
            b"abcdefghijklmnopqrstuvwxyz"
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"
            b"0123456789!@#0^&*();:<>,. []{}",
        )
        eq(base64.decodebytes(b""), b"")
        # Non-bytes
        eq(base64.decodebytes(bytearray(b"YWJj")), b"abc")
        # eq(base64.decodebytes(memoryview(b"YWJj\n")), b"abc")
        # eq(base64.decodebytes(array('B', b'YWJj\n')), b'abc')
        # self.check_type_errors(base64.decodebytes)


if __name__ == "__main__":
    unittest.main()
