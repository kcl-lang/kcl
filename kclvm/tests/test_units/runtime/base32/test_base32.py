import kclvm_runtime
import os
import tempfile
import subprocess
import unittest
import sys

# Add the parent directory to the path to import kclvm_runtime
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.append(parent_dir)

_Dylib = kclvm_runtime.KclvmRuntimeDylib()


class kclx_Base32:
    def __init__(self, dylib_=None):
        self.dylib = dylib_ if dylib_ else _Dylib

    def encode(self, value: str) -> str:
        return self.dylib.Invoke("base32.encode", value)

    def decode(self, value: str) -> str:
        return self.dylib.Invoke("base32.decode", value)


base32 = kclx_Base32(_Dylib)


class BaseTest(unittest.TestCase):
    def test_encode(self):
        # Test vectors from RFC 4648
        test_cases = [
            ("", ""),
            ("f", "MY======"),
            ("fo", "MZXQ===="),
            ("foo", "MZXW6==="),
            ("foob", "MZXW6YQ="),
            ("fooba", "MZXW6YTB"),
            ("foobar", "MZXW6YTBOI======"),
        ]

        for input_str, expected_output in test_cases:
            result = base32.encode(input_str)
            self.assertEqual(expected_output, result)

    def test_decode(self):
        # Test vectors from RFC 4648
        test_cases = [
            ("", ""),
            ("MY======", "f"),
            ("MZXQ====", "fo"),
            ("MZXW6===", "foo"),
            ("MZXW6YQ=", "foob"),
            ("MZXW6YTB", "fooba"),
            ("MZXW6YTBOI======", "foobar"),
        ]

        for input_str, expected_output in test_cases:
            result = base32.decode(input_str)
            self.assertEqual(expected_output, result)

    def test_encode_decode(self):
        # Test that encoding and then decoding gives the original string
        test_strings = [
            "",
            "hello world",
            "0.3.0",
            "special chars: !@#$%^&*()",
            "unicode: 你好世界",
        ]

        for test_str in test_strings:
            encoded = base32.encode(test_str)
            decoded = base32.decode(encoded)
            self.assertEqual(test_str, decoded)


if __name__ == "__main__":
    unittest.main()
