# Copyright 2020 The KCL Authors. All rights reserved.

import sys
import typing
import unittest

import kclvm.kcl.error as kcl_error
import kclvm.api.object.internal.decorators as decorators


targets = [
    decorators.DecoratorTargetType.SCHEMA_TYPE,
    decorators.DecoratorTargetType.ATTRIBUTE,
]


class TestDecoratorFactory(unittest.TestCase):
    def test_decorator_factory_normal(self):
        for target in targets:
            decorators.decorator_factory.get(decorators.Deprecated.NAME, target)

    def test_decorator_factory_invalid(self):
        for target in targets:
            with self.assertRaises(kcl_error.UnKnownDecoratorError) as err:
                decorators.decorator_factory.get(None, target)
            self.assertEqual(err.exception.ewcode, kcl_error.ErrEwcode.UnKnownDecorator_Ew)
            self.assertEqual(err.exception.arg_msg, "UnKnown decorator ")


class TestDecoratorDeprecated(unittest.TestCase):
    def test_deprecated_schema(self):
        decorator = decorators.Deprecated(
            decorators.Deprecated.NAME, decorators.DecoratorTargetType.SCHEMA_TYPE
        )
        with self.assertRaises(kcl_error.DeprecatedError) as err:
            decorator.run(key="key", value="value")
        self.assertEqual(err.exception.ewcode, kcl_error.ErrEwcode.Deprecated_Ew)
        self.assertEqual(str(err.exception.arg_msg), "key was deprecated ")

    def test_deprecated_attr(self):
        decorator = decorators.Deprecated(
            decorators.Deprecated.NAME, decorators.DecoratorTargetType.ATTRIBUTE
        )
        with self.assertRaises(kcl_error.DeprecatedError) as err:
            decorator.run(key="key", value="value")
        self.assertEqual(err.exception.ewcode, kcl_error.ErrEwcode.Deprecated_Ew)
        self.assertEqual(str(err.exception.arg_msg), "key was deprecated ")

    def test_deprecated_attr_without_value(self):
        decorator = decorators.Deprecated(
            decorators.Deprecated.NAME, decorators.DecoratorTargetType.ATTRIBUTE
        )
        decorator.run(key="key", value=None)

    def test_deprecated_invalid_target(self):
        with self.assertRaises(kcl_error.InvalidDecoratorTargetError) as err:
            decorators.Deprecated(decorators.Deprecated.NAME, None)
        self.assertEqual(err.exception.ewcode, kcl_error.ErrEwcode.InvalidDecoratorTarget_Ew)
        self.assertEqual(str(err.exception.arg_msg), "Invalid decorator target ")

    def test_deprecated_invalid_key(self):
        for target in targets:
            with self.assertRaises(kcl_error.KCLNameError) as err:
                decorators.Deprecated(decorators.Deprecated.NAME, target).run("", "")
            self.assertEqual(err.exception.ewcode, kcl_error.ErrEwcode.KCLNameError_Ew)
            self.assertEqual(err.exception.arg_msg, "Name error : Decorator target name cannot be empty")

    def test_deprecated_version_parameter(self):
        for target in targets:
            decorator = decorators.Deprecated(
                decorators.Deprecated.NAME,
                target,
                version="v1.16",
            )
            with self.assertRaises(kcl_error.DeprecatedError) as err:
                decorator.run(key="key", value="value")
            self.assertEqual(err.exception.ewcode, kcl_error.ErrEwcode.Deprecated_Ew)
            self.assertEqual(
                str(err.exception.arg_msg), "key was deprecated since version v1.16"
            )

    def test_deprecated_version_reason_parameter(self):
        for target in targets:
            decorator = decorators.Deprecated(
                decorators.Deprecated.NAME,
                target,
                reason="key is not supported",
                version="v1.16",
            )
            with self.assertRaises(kcl_error.DeprecatedError) as err:
                decorator.run(key="key", value="value")
            self.assertEqual(err.exception.ewcode, kcl_error.ErrEwcode.Deprecated_Ew)
            self.assertEqual(
                str(err.exception.arg_msg),
                "key was deprecated since version v1.16, key is not supported",
            )

    def test_deprecated_version_strict_parameter(self):
        for target in targets:
            decorator = decorators.Deprecated(
                decorators.Deprecated.NAME,
                target,
                strict=False,
            )
            decorator.run(key="key", value="value")


if __name__ == "__main__":
    unittest.main(verbosity=2)
