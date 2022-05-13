# Copyright 2020 The KCL Authors. All rights reserved.

import unittest

import kclvm.config
import kclvm.api.object.internal.option as option


class TestOption(unittest.TestCase):
    def setUp(self):
        kclvm.config.arguments = [
            ("key1", "value1"),
            ("key1", "value2"),
            ("key2", "s"),
            ("key3", 1),
            ("key4", 1.0),
            ("key5", True),
            ("key6", False),
            ("key7", "1"),
            ("key8", "1.0"),
        ]
        option.kcl_option_init_all()
        return super().setUp()

    def tearDown(self):
        kclvm.config.arguments = []
        option.kcl_option_reset()
        return super().tearDown()

    def test_kcl_option_elem(self):
        elem = option.KclOptionElem("key")
        self.assertEqual(str(elem), "key=?")
        elem.default = "value"
        self.assertEqual(str(elem), "key=value")
        elem.value_type = "str"
        self.assertEqual(str(elem), "key=value (str)")
        elem.required = True
        self.assertEqual(str(elem), "key=value (str,required)")
        elem.value_type = ""
        self.assertEqual(str(elem), "key=value (required)")
        elem.file = "main.k"
        elem.line = 1
        self.assertEqual(elem.get_help(verbose_mode=2), "key=value (required)  (main.k:1)")

    def test_kcl_option_dict(self):
        elem_key1 = option.KclOptionElem("key1")
        elem_key2 = option.KclOptionElem("key2")
        option_dict = option._KclOptionDict()
        self.assertEqual(option_dict.help(), "")
        option_dict.m["key1"] = elem_key1
        option_dict.m["key2"] = elem_key2
        self.assertEqual(option_dict.len(), 2)
        self.assertEqual(option_dict.get_dict(), option_dict.m)
        self.assertEqual(option_dict.keys(), ["key1", "key2"])
        self.assertEqual(option_dict.has_key("key1"), True)
        self.assertEqual(option_dict.has_key("key_err"), False)
        self.assertEqual(option_dict.help(), "option list:\nkey1=?\nkey2=?")

    def test_option_not_exist(self):
        self.assertEqual(option.kcl_option("not_exist_key"), None)
        self.assertEqual(option.kcl_option("not_exist_key", default=1), 1)
        self.assertEqual(option.kcl_option("not_exist_key", default=1.0), 1.0)
        self.assertEqual(option.kcl_option("not_exist_key", default=True), True)

    def test_option(self):
        self.assertEqual(option.kcl_option("key1"), "value2")
        self.assertEqual(option.kcl_option("key2"), "s")
        self.assertEqual(option.kcl_option("key3"), 1)
        self.assertEqual(option.kcl_option("key4"), 1.0)
        self.assertEqual(option.kcl_option("key5"), True)
        self.assertEqual(option.kcl_option("key6"), False)
        self.assertEqual(option.kcl_option("key7", type="int"), 1)
        self.assertEqual(option.kcl_option("key8", type="float"), 1.0)


if __name__ == "__main__":
    unittest.main(verbosity=2)
