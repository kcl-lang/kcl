# Copyright 2021 The KCL Authors. All rights reserved.

import sys
import enum
from abc import ABCMeta, abstractmethod
from typing import Dict, Type

import kclvm.kcl.error as kcl_error

DECORATOR_TARGET_ERR_NAME_MSG = ": Decorator target name cannot be empty"


class DecoratorTargetType(enum.Enum):
    """
    Marked annotated position by the decorator
    """

    SCHEMA_TYPE = 0
    ATTRIBUTE = 1


class Decorator(metaclass=ABCMeta):
    """
    An abstract decorator.

    This abc is used to run actions as a wrapper of key-value pair in kcl schema.

    A concrete decorator should inherit this class and impel the run method to handle key-value.

    :class:`~Deprecated` class is a sample of decorator which is used to check deprecation of key on config.

    you can use your ``Deprecated`` class like this in your source code as follows:

    .. code-block:: python

        schema Person:
            @deprecated(version="1.16", reason="use firstName and lastName instead", strict=True)
            name : str

    .. code-block:: python

        @deprecated(version="1.16", reason="use firstName and lastName instead", strict=True)
        schema Person:
            name : str

    """

    def __init__(self, name: str, target: DecoratorTargetType, *args, **kwargs):
        self.name = name
        try:
            self.target = (
                target
                if isinstance(target, DecoratorTargetType)
                else DecoratorTargetType(target)
            )
        except ValueError:
            msg = (
                kcl_error.INVALID_DECORATOR_TARGET_MSG.format(target)
                if target
                else kcl_error.INVALID_DECORATOR_TARGET_MSG.format("")
            )
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.InvalidDecoratorTarget_TYPE,
                file_msgs=[
                    kcl_error.ErrFileMsg(
                        filename=kwargs.get("filename"),
                        line_no=kwargs.get("lineno"),
                        col_no=kwargs.get("columnno"),
                    )
                ],
                arg_msg=msg,
            )
        self.filename = kwargs.get("filename")
        self.lineno = kwargs.get("lineno")
        self.columnno = kwargs.get("columnno")

    @abstractmethod
    def run(self, key: str, value, *args, **kwargs):
        """
        Decorate on a key-value pair in runtime.

        :param key: Key to Decorated
        :param value: Value to Decorated
        :param args: Decorator run positional args
        :param kwargs: Decorator run keyword args
        :return:
        """
        pass


class DecoratorFactory:
    """
    A decorator factory used to get decorator object.
    """

    def __init__(self):
        self._decorators: Dict[str, Type[Decorator]] = {}

    def register(self, name: str, decorator: Type[Decorator]):
        """
        Register a decorator with a unique name.

        :param name: Name of the decorator
        :param decorator: The decorator to be registered
        :return: None
        """
        self._decorators[name] = decorator

    def get(self, name: str, target: DecoratorTargetType, *args, **kwargs):
        """
        Get and return a decorator object.
        An UnKnownDecorator will be thrown if no decorator found.

        :param name: Name of the decorator
        :param target: Target of the decorator e.g., schema and attribute
        :param args: Decorator meta info positional args
        :param kwargs: Decorator meta info keyword args
        :return: A decorator object
        """
        decorator = self._decorators.get(name)
        if not decorator:
            filename = kwargs.get("filename")
            lineno = kwargs.get("lineno")
            columnno = kwargs.get("columnno")
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.UnKnownDecorator_TYPE,
                file_msgs=[
                    kcl_error.ErrFileMsg(
                        filename=filename, line_no=lineno, col_no=columnno
                    )
                ],
                arg_msg=kcl_error.UNKNOWN_DECORATOR_MSG.format(name)
                if name
                else kcl_error.UNKNOWN_DECORATOR_MSG.format(""),
            )
        return decorator(name, target, *args, **kwargs)


class Deprecated(Decorator):
    """This decorator is used to get the deprecation message according to the wrapped key-value pair.

    Examples
    --------
        @deprecated(version="v1.16", reason="The age attribute was deprecated", strict=True)
        schema Person:
            name: str
            age: int
    """

    NAME = "deprecated"

    def __init__(self, name, target, *args, **kwargs):
        """
        Construct a deprecated decorator

        :param args: Deprecated decorator build positional args
        :param kwargs: Deprecated decorator build keyword args
        """
        super().__init__(self.NAME, target, *args, **kwargs)
        self.version = kwargs.get("version", "")
        self.reason = kwargs.get("reason", "")
        self.strict = kwargs.get("strict", True)

    def run(self, key: str, value, *args, **kwargs):
        """
        Build and report deprecation message.

        A KCL runtime error will be thrown if self.strict is True, otherwise it just print warning message.

        :param key: Key to Deprecated decorated
        :param value: Value to Deprecated decorated
        :param args: Deprecated decorator run positional args
        :param kwargs: Deprecated decorator run positional args
        :return should_change_value: Mark the value of the deprecated schema attribute should be modified
        """
        filename = self.filename
        lineno = self.lineno
        columnno = self.columnno
        if not key:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.NameError_TYPE,
                file_msgs=[
                    kcl_error.ErrFileMsg(
                        filename=filename, line_no=lineno, col_no=columnno
                    )
                ],
                arg_msg=kcl_error.NAME_ERROR_MSG.format(DECORATOR_TARGET_ERR_NAME_MSG),
            )
        # Mark the value of the deprecated schema attribute should be modified
        should_change_value = False
        # Error or warning message
        msg = ""
        # Append a version info into message
        if self.version:
            msg += "since version {}".format(self.version)
        # Append a reason info into message
        if self.reason:
            msg += ", " + self.reason
        if self.strict:
            if self.target == DecoratorTargetType.SCHEMA_TYPE or (
                self.target == DecoratorTargetType.ATTRIBUTE and value
            ):
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.Deprecated_TYPE,
                    file_msgs=[
                        kcl_error.ErrFileMsg(
                            filename=filename, line_no=lineno, col_no=columnno
                        )
                    ],
                    arg_msg=kcl_error.DEPRECATED_WARNING_MSG.format(key, msg),
                )
            should_change_value = True
        else:
            # If it is a modified schema attribute, ignore the assignment without reporting an error.
            kcl_error.print_kcl_warning_message(
                kcl_error.get_exception(
                    err_type=kcl_error.ErrType.Deprecated_Warning_TYPE,
                    file_msgs=[
                        kcl_error.ErrFileMsg(
                            filename=filename, line_no=lineno, col_no=columnno
                        )
                    ],
                    arg_msg=kcl_error.DEPRECATED_WARNING_MSG.format(key, msg),
                ),
                file=sys.stderr,
            )
            should_change_value = False
        return should_change_value


class Info(Decorator):
    """Info decorator is used to mark some compile-time information for external API queries

    Examples
    --------
        @info(message="User message")
        schema Person:
            name: str
            age: int
    """

    NAME = "info"

    def __init__(self, name, target, *args, **kwargs):
        """Construct a Info decorator"""
        super().__init__(self.NAME, target, *args, **kwargs)

    def run(self, key: str, value, *args, **kwargs):
        """Nothing to do on Info decorator"""
        pass


decorator_factory = DecoratorFactory()
decorator_factory.register(Deprecated.NAME, Deprecated)
decorator_factory.register(Info.NAME, Info)
