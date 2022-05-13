# Copyright 2020 The KCL Authors. All rights reserved.

import unittest

from kclvm.api.object.internal import Undefined


class TestUndefinedObject(unittest.TestCase):
    def test_undefined_object(self):
        self.assertEqual(str(Undefined), "Undefined")
        self.assertEqual(repr(Undefined), "Undefined")
        self.assertEqual(Undefined.type_str(), "UndefinedType")
        self.assertEqual(bool(Undefined), False)
        self.assertEqual(not Undefined, True)
        self.assertEqual(Undefined.value, None)


if __name__ == "__main__":
    unittest.main(verbosity=2)
