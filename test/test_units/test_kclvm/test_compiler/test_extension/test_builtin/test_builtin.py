# Copyright 2020 The KCL Authors. All rights reserved.

import unittest

import kclvm.compiler.extension.builtin.builtin as builtin


class TestBuiltinFcuntion(unittest.TestCase):
    def test_list(self):
        self.assertEqual(builtin.KMANGLED_list(), [])
        self.assertEqual(builtin.KMANGLED_list([0]), [0])
        self.assertEqual(builtin.KMANGLED_list({"k": "v"}), ["k"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
