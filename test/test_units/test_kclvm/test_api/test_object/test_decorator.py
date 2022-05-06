# Copyright 2020 The KCL Authors. All rights reserved.

import unittest

import kclvm.kcl.error as kcl_error
import kclvm.api.object as objpkg

from kclvm.api.object.internal import Decorator


class TestDecoratorObject(unittest.TestCase):
    def test_decorator_deprecated(self):
        schema_name = "Person"
        deprecated_decorator_obj = objpkg.KCLDecoratorObject(
            target=objpkg.DecoratorTargetType.SCHEMA_TYPE,
            name="deprecated",
            key=schema_name,
        )
        deprecated_decorator_obj.resolve([], [])
        self.assertEqual(
            deprecated_decorator_obj.type(), objpkg.KCLObjectType.DECORATOR
        )
        self.assertEqual(deprecated_decorator_obj.type_str(), "decorator")
        self.assertIsInstance(deprecated_decorator_obj.decorator, Decorator)
        with self.assertRaises(kcl_error.DeprecatedError) as err:
            deprecated_decorator_obj.call([], [], key=schema_name)
        self.assertIn(f"{schema_name} was deprecated", str(err.exception))


if __name__ == "__main__":
    unittest.main(verbosity=2)
