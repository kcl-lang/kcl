#!/usr/bin/env python3
# -*- coding: UTF-8 -*-

import os
import unittest

import kclvm.kcl.info as kcl_info


class TestKCLInfoNaming(unittest.TestCase):
    def test_mangle(self):
        cases = [
            {"name": "", "expected": "KMANGLED_"},
            {"name": "a", "expected": "KMANGLED_a"},
            {"name": "a.b", "expected": "KMANGLED_a.KMANGLED_b"},
        ]
        for case in cases:
            name, expected = case["name"], case["expected"]
            self.assertEqual(kcl_info.mangle(name), expected)

    def test_demangle(self):
        cases = [
            {"name": "", "expected": ""},
            {"name": "KMANGLED_a", "expected": "a"},
            {"name": "KMANGLED_a.KMANGLED_b", "expected": "a.b"},
        ]
        for case in cases:
            name, expected = case["name"], case["expected"]
            self.assertEqual(kcl_info.demangle(name), expected)


    def test_tagging(self):
        cases = [
            {"name": "", "tag": "", "expected": "KTAG__"},
            {"name": "", "tag": "attr", "expected": "KTAG_attr_"},
            {"name": "a", "tag": "attr", "expected": "KTAG_attr_a"},
            {"name": "a.b", "tag": "attr", "expected": "KTAG_attr_a.b"},
        ]
        for case in cases:
            name, tag, expected = case["name"], case["tag"], case["expected"]
            self.assertEqual(kcl_info.tagging(tag, name), expected)

    def test_detagging(self):
        cases = [
            {"name": "", "tag": "", "expected": ""},
            {"name": "KTAG_attr_", "tag": "attr", "expected": ""},
            {"name": "KTAG_attr_a", "tag": "attr", "expected": "a"},
            {"name": "KTAG_attr_a.b", "tag": "attr", "expected": "a.b"},
        ]
        for case in cases:
            name, tag, expected = case["name"], case["tag"], case["expected"]
            self.assertEqual(kcl_info.detagging(tag, name), expected)


if __name__ == "__main__":
    unittest.main(verbosity=2)
