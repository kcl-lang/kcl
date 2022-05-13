# Copyright 2021 The KCL Authors. All rights reserved.

import typing
import unittest

import kclvm_runtime

# https://github.com/python/cpython/blob/main/Lib/test/test_re.py

# kclvm_regex_compile,
# kclvm_regex_findall,
# kclvm_regex_match,
# kclvm_regex_replace,
# kclvm_regex_search,
# kclvm_regex_split,

_Dylib = kclvm_runtime.KclvmRuntimeDylib()


class kclx_Regex:
    def __init__(self, dylib_=None):
        self.dylib = dylib_ if dylib_ else _Dylib

    def compile(self, pattern: str) -> bool:
        return self._kcl_compile(f"{pattern}")

    def match(self, pattern: str, string: str) -> bool:
        return self._kcl_match(f"{string}", f"{pattern}")

    def compile(self, pattern: str) -> bool:
        return self._kcl_compile(f"{pattern}")

    def findall(self, pattern: str, string: str) -> typing.List[str]:
        return self._kcl_findall(f"{string}", f"{pattern}")

    def search(self, pattern: str, string: str) -> bool:
        return self._kcl_search(f"{string}", f"{pattern}")

    def split(self, pattern: str, string: str) -> typing.List[str]:
        return self._kcl_split(f"{string}", f"{pattern}")

    def _kcl_match(self, string: str, pattern: str) -> bool:
        return self.dylib.Invoke(f"regex.match", string, pattern)

    def _kcl_replace(
        self, string: str, pattern: str, replace: str = None, count: int = 0
    ) -> str:
        return self.dylib.Invoke(f"regex.replace", string, pattern, replace, count)

    def _kcl_compile(self, pattern: str) -> bool:
        return self.dylib.Invoke(f"regex.compile", pattern)

    def _kcl_findall(self, string, pattern: str) -> typing.List[str]:
        return self.dylib.Invoke(f"regex.findall", string, pattern)

    def _kcl_search(self, string: str, pattern: str) -> bool:
        return self.dylib.Invoke(f"regex.search", string, pattern)

    def _kcl_split(
        self, string: str, pattern: str, maxsplit: int = 0
    ) -> typing.List[str]:
        return self.dylib.Invoke(f"regex.split", string, pattern, maxsplit)


re = kclx_Regex(_Dylib)


class S(str):
    def __getitem__(self, index):
        return S(super().__getitem__(index))


class B(bytes):
    def __getitem__(self, index):
        return B(super().__getitem__(index))


class BaseTest(unittest.TestCase):
    def test_match_konfig_case(self):
        # error: look-around, including look-ahead and look-behind, is not supported
        self.assertTrue(re.match(r"^(?!-)[a-z0-9-]{1,63}(?<!-)$", "my-service-name"))

        self.assertFalse(
            re.match(
                "^(1\\d{2}|2[0-4]\\d|25[0-5]|[1-9]\\d|[1-9])\\."
                + "(1\\d{2}|2[0-4]\\d|25[0-5]|[1-9]\\d|\\d)\\."
                + "(1\\d{2}|2[0-4]\\d|25[0-5]|[1-9]\\d|\\d)\\."
                + "(1\\d{2}|2[0-4]\\d|25[0-5]|[1-9]\\d|\\d)$",
                "192.168.0,1",
            )
        )

    def test_search_star_plus(self):
        self.assertTrue(re.search("x*", "axx"))
        self.assertTrue(re.search("x*", "axx"))
        self.assertTrue(re.search("x+", "axx"))
        self.assertTrue(re.search("x+", "axx"))
        self.assertFalse(re.search("x", "aaa"))
        self.assertTrue(re.match("a*", "xxx"))
        self.assertTrue(re.match("a*", "xxx"))
        self.assertTrue(re.match("x*", "xxxa"))
        self.assertTrue(re.match("x*", "xxxa"))
        self.assertFalse(re.match("a+", "xxx"))

    def test_re_split(self):
        for string in ":a:b::c", S(":a:b::c"):
            self.assertEqual(re.split(":", string), ["", "a", "b", "", "c"])
            self.assertEqual(re.split(":+", string), ["", "a", "b", "c"])
            # self.assertTypedEqual(
            #    re.split("(:+)", string), ["", ":", "a", ":", "b", "::", "c"]
            # )

    def test_re_findall(self):
        self.assertEqual(re.findall(":+", "abc"), [])
        self.assertEqual(
            re.findall(r"^(\d{0,63})(E|P|T|G|M|K|Ei|Pi|Ti|Gi|Mi|Ki)$", "4Gi"),
            [["4", "Gi"]],
        )
        for string in "a:b::c:::d", S("a:b::c:::d"):
            self.assertEqual(re.findall(":+", string), [":", "::", ":::"])
            self.assertEqual(re.findall("(:+)", string), [":", "::", ":::"])
            self.assertEqual(
                re.findall("(:)(:*)", string), [[":", ""], [":", ":"], [":", "::"]]
            )

        # for string in (
        #    b"a:b::c:::d",
        #    B(b"a:b::c:::d"),
        # ):
        #    self.assertEqual(re.findall(b":+", string), [b":", b"::", b":::"])
        #    self.assertEqual(re.findall(b"(:+)", string), [b":", b"::", b":::"])
        #    self.assertEqual(
        #        re.findall(b"(:)(:*)", string),
        #        [(b":", b""), (b":", b":"), (b":", b"::")],
        #    )
        for x in ("\xe0", "\u0430", "\U0001d49c"):
            xx = x * 2
            xxx = x * 3
            string = "a%sb%sc%sd" % (x, xx, xxx)
            self.assertEqual(re.findall("%s+" % x, string), [x, xx, xxx])
            self.assertEqual(re.findall("(%s+)" % x, string), [x, xx, xxx])
            self.assertEqual(
                re.findall("(%s)(%s*)" % (x, x), string), [[x, ""], [x, x], [x, xx]]
            )

    def test_repeat_minmax(self):
        self.assertFalse(re.match(r"^(\w){1}$", "abc"))
        self.assertFalse(re.match(r"^(\w){1}?$", "abc"))
        self.assertFalse(re.match(r"^(\w){1,2}$", "abc"))
        self.assertFalse(re.match(r"^(\w){1,2}?$", "abc"))

        self.assertTrue(re.match(r"^x{3}$", "xxx"))
        self.assertTrue(re.match(r"^x{1,3}$", "xxx"))
        self.assertTrue(re.match(r"^x{3,3}$", "xxx"))
        self.assertTrue(re.match(r"^x{1,4}$", "xxx"))
        self.assertTrue(re.match(r"^x{3,4}?$", "xxx"))
        self.assertTrue(re.match(r"^x{3}?$", "xxx"))
        self.assertTrue(re.match(r"^x{1,3}?$", "xxx"))
        self.assertTrue(re.match(r"^x{1,4}?$", "xxx"))
        self.assertTrue(re.match(r"^x{3,4}?$", "xxx"))

        self.assertFalse(re.match(r"^x{}$", "xxx"))
        self.assertTrue(re.match(r"^x{}$", "x{}"))

    def _test_named_unicode_escapes(self):
        # test individual Unicode named escapes
        self.assertTrue(re.match(r"\N{LESS-THAN SIGN}", "<"))
        self.assertTrue(re.match(r"\N{less-than sign}", "<"))
        self.assertFalse(re.match(r"\N{LESS-THAN SIGN}", ">"))
        self.assertTrue(re.match(r"\N{SNAKE}", "\U0001f40d"))
        self.assertTrue(
            re.match(
                r"\N{ARABIC LIGATURE UIGHUR KIRGHIZ YEH WITH "
                r"HAMZA ABOVE WITH ALEF MAKSURA ISOLATED FORM}",
                "\ufbf9",
            )
        )
        self.assertTrue(re.match(r"[\N{LESS-THAN SIGN}-\N{GREATER-THAN SIGN}]", "="))
        self.assertFalse(re.match(r"[\N{LESS-THAN SIGN}-\N{GREATER-THAN SIGN}]", ";"))

    def test_string_boundaries(self):

        # There's a word boundary at the start of a string.
        self.assertTrue(re.match(r"\b", "abc"))
        # A non-empty string includes a non-boundary zero-length match.
        self.assertTrue(re.search(r"\B", "abc"))
        # There is no non-boundary match at the start of a string.
        # self.assertFalse(re.match(r"\B", "abc"))
        # However, an empty string contains no word boundaries, and also no
        # non-boundaries.
        # self.assertFalse(re.search(r"\B", ""))
        # This one is questionable and different from the perlre behaviour,
        # but describes current behavior.
        self.assertFalse(re.search(r"\b", ""))

    def test_big_codesize(self):
        # Issue #1160
        r = "|".join(("%d" % x for x in range(10000)))
        self.assertTrue(re.match(r, "1000"))
        self.assertTrue(re.match(r, "9999"))

    def _test_lookahead(self):

        # Group reference.
        self.assertTrue(re.match(r"(a)b(?=\1)a", "aba"))
        self.assertFalse(re.match(r"(a)b(?=\1)c", "abac"))
        # Conditional group reference.
        self.assertTrue(re.match(r"(?:(a)|(x))b(?=(?(2)x|c))c", "abc"))
        self.assertFalse(re.match(r"(?:(a)|(x))b(?=(?(2)c|x))c", "abc"))
        self.assertTrue(re.match(r"(?:(a)|(x))b(?=(?(2)x|c))c", "abc"))
        self.assertFalse(re.match(r"(?:(a)|(x))b(?=(?(1)b|x))c", "abc"))
        self.assertTrue(re.match(r"(?:(a)|(x))b(?=(?(1)c|x))c", "abc"))
        # Group used before defined.
        self.assertTrue(re.match(r"(a)b(?=(?(2)x|c))(c)", "abc"))
        self.assertFalse(re.match(r"(a)b(?=(?(2)b|x))(c)", "abc"))
        self.assertTrue(re.match(r"(a)b(?=(?(1)c|x))(c)", "abc"))

    def _test_lookbehind(self):
        self.assertTrue(re.match(r"ab(?<=b)c", "abc"))
        self.assertFalse(re.match(r"ab(?<=c)c", "abc"))
        self.assertFalse(re.match(r"ab(?<!b)c", "abc"))
        self.assertTrue(re.match(r"ab(?<!c)c", "abc"))
        # Group reference.
        self.assertTrue(re.match(r"(a)a(?<=\1)c", "aac"))
        self.assertFalse(re.match(r"(a)b(?<=\1)a", "abaa"))
        self.assertFalse(re.match(r"(a)a(?<!\1)c", "aac"))
        self.assertTrue(re.match(r"(a)b(?<!\1)a", "abaa"))
        # Conditional group reference.
        self.assertFalse(re.match(r"(?:(a)|(x))b(?<=(?(2)x|c))c", "abc"))
        self.assertFalse(re.match(r"(?:(a)|(x))b(?<=(?(2)b|x))c", "abc"))
        self.assertTrue(re.match(r"(?:(a)|(x))b(?<=(?(2)x|b))c", "abc"))
        self.assertFalse(re.match(r"(?:(a)|(x))b(?<=(?(1)c|x))c", "abc"))
        self.assertTrue(re.match(r"(?:(a)|(x))b(?<=(?(1)b|x))c", "abc"))
        # Group used before defined.
        # self.assertRaises(re.error, re.compile, r'(a)b(?<=(?(2)b|x))(c)')
        self.assertFalse(re.match(r"(a)b(?<=(?(1)c|x))(c)", "abc"))
        self.assertTrue(re.match(r"(a)b(?<=(?(1)b|x))(c)", "abc"))

    def _test_sre_character_literals(self):
        for i in [0, 8, 16, 32, 64, 127, 128, 255, 256, 0xFFFF, 0x10000, 0x10FFFF]:
            if i < 256:
                self.assertTrue(re.match(r"\%03o" % i, chr(i)))
                self.assertTrue(re.match(r"\%03o0" % i, chr(i) + "0"))
                self.assertTrue(re.match(r"\%03o8" % i, chr(i) + "8"))
                self.assertTrue(re.match(r"\x%02x" % i, chr(i)))
                self.assertTrue(re.match(r"\x%02x0" % i, chr(i) + "0"))
                self.assertTrue(re.match(r"\x%02xz" % i, chr(i) + "z"))
            if i < 0x10000:
                self.assertTrue(re.match(r"\u%04x" % i, chr(i)))
                self.assertTrue(re.match(r"\u%04x0" % i, chr(i) + "0"))
                self.assertTrue(re.match(r"\u%04xz" % i, chr(i) + "z"))
            self.assertTrue(re.match(r"\U%08x" % i, chr(i)))
            self.assertTrue(re.match(r"\U%08x0" % i, chr(i) + "0"))
            self.assertTrue(re.match(r"\U%08xz" % i, chr(i) + "z"))
        self.assertTrue(re.match(r"\0", "\000"))
        self.assertTrue(re.match(r"\08", "\0008"))
        self.assertTrue(re.match(r"\01", "\001"))
        self.assertTrue(re.match(r"\018", "\0018"))

    def _test_sre_character_class_literals(self):
        for i in [0, 8, 16, 32, 64, 127, 128, 255, 256, 0xFFFF, 0x10000, 0x10FFFF]:
            if i < 256:
                self.assertTrue(re.match(r"[\%o]" % i, chr(i)))
                self.assertTrue(re.match(r"[\%o8]" % i, chr(i)))
                self.assertTrue(re.match(r"[\%03o]" % i, chr(i)))
                self.assertTrue(re.match(r"[\%03o0]" % i, chr(i)))
                self.assertTrue(re.match(r"[\%03o8]" % i, chr(i)))
                self.assertTrue(re.match(r"[\x%02x]" % i, chr(i)))
                self.assertTrue(re.match(r"[\x%02x0]" % i, chr(i)))
                self.assertTrue(re.match(r"[\x%02xz]" % i, chr(i)))
            if i < 0x10000:
                self.assertTrue(re.match(r"[\u%04x]" % i, chr(i)))
                self.assertTrue(re.match(r"[\u%04x0]" % i, chr(i)))
                self.assertTrue(re.match(r"[\u%04xz]" % i, chr(i)))
            self.assertTrue(re.match(r"[\U%08x]" % i, chr(i)))
            self.assertTrue(re.match(r"[\U%08x0]" % i, chr(i) + "0"))
            self.assertTrue(re.match(r"[\U%08xz]" % i, chr(i) + "z"))

    def test_search_dot_unicode(self):
        self.assertTrue(re.search("123.*-", "123abc-"))
        self.assertTrue(re.search("123.*-", "123\xe9-"))
        self.assertTrue(re.search("123.*-", "123\u20ac-"))
        # self.assertTrue(re.search("123.*-", "123\U0010ffff-"))
        # self.assertTrue(re.search("123.*-", "123\xe9\u20ac\U0010ffff-"))

    def check_en_US_iso88591(self):
        # locale.setlocale(locale.LC_CTYPE, 'en_US.iso88591')
        # self.assertTrue(re.match(b'\xc5\xe5', b'\xc5\xe5', re.L|re.I))
        # self.assertTrue(re.match(b'\xc5', b'\xe5', re.L|re.I))
        # self.assertTrue(re.match(b'\xe5', b'\xc5', re.L|re.I))
        self.assertTrue(re.match(b"(?Li)\xc5\xe5", b"\xc5\xe5"))
        self.assertTrue(re.match(b"(?Li)\xc5", b"\xe5"))
        self.assertTrue(re.match(b"(?Li)\xe5", b"\xc5"))

    def check_en_US_utf8(self):
        # locale.setlocale(locale.LC_CTYPE, 'en_US.utf8')
        # self.assertTrue(re.match(b'\xc5\xe5', b'\xc5\xe5', re.L|re.I))
        # self.assertIsNone(re.match(b'\xc5', b'\xe5', re.L|re.I))
        # self.assertIsNone(re.match(b'\xe5', b'\xc5', re.L|re.I))
        self.assertTrue(re.match(b"(?Li)\xc5\xe5", b"\xc5\xe5"))
        self.assertIsNone(re.match(b"(?Li)\xc5", b"\xe5"))
        self.assertIsNone(re.match(b"(?Li)\xe5", b"\xc5"))


if __name__ == "__main__":
    unittest.main()
