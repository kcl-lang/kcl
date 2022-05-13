# Copyright 2021 The KCL Authors. All rights reserved.

import typing
import unittest

import kclvm_runtime

# md5(value: str, encoding: str = "utf-8") -> str
# sha1(value: str, encoding: str = "utf-8") -> str
# sha224(value: str, encoding: str = "utf-8") -> str
# sha256(value: str, encoding: str = "utf-8") -> str
# sha384(value: str, encoding: str = "utf-8") -> str
# sha512(value: str, encoding: str = "utf-8") -> str

_Dylib = kclvm_runtime.KclvmRuntimeDylib()


class BaseTest(unittest.TestCase):
    def __init__(self, methodName="runTest"):
        super().__init__(methodName)
        self.dylib = _Dylib

    def md5(self, value: str) -> str:
        return self.dylib.Invoke(f"crypto.md5", value)

    def sha1(self, value: str) -> str:
        return self.dylib.Invoke(f"crypto.sha1", value)

    def sha224(self, value: str) -> str:
        return self.dylib.Invoke(f"crypto.sha224", value)

    def sha256(self, value: str) -> str:
        return self.dylib.Invoke(f"crypto.sha256", value)

    def sha384(self, value: str) -> str:
        return self.dylib.Invoke(f"crypto.sha384", value)

    def sha512(self, value: str) -> str:
        return self.dylib.Invoke(f"crypto.sha512", value)

    def test_md5(self):
        self.assertEqual(
            self.md5("The quick brown fox jumps over the lazy dog"),
            "9e107d9d372bb6826bd81d3542a419d6",
        )
        self.assertEqual(
            self.md5("The quick brown fox jumps over the lazy cog"),
            "1055d3e698d289f2af8663725127bd4b",
        )
        self.assertEqual(
            self.md5(""),
            "d41d8cd98f00b204e9800998ecf8427e",
        )

    def test_sha1(self):
        self.assertEqual(self.sha1(""), "da39a3ee5e6b4b0d3255bfef95601890afd80709")

    def test_sha224(self):
        self.assertEqual(
            self.sha224(""), "d14a028c2a3a2bc9476102bb288234c415a2b01f828ea62ac5b3e42f"
        )

    def test_sha256(self):
        self.assertEqual(
            self.sha256(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        )

    def test_sha384(self):
        self.assertEqual(
            self.sha384(""),
            "38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b",
        )

    def test_sha512(self):
        self.assertEqual(
            self.sha512(""),
            "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
        )


if __name__ == "__main__":
    unittest.main()
