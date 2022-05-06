"""
The `kcl_err_theme` file mainly contains the color constants needed to highlight the KCL error message.

ThemeId (Enum): The color theme of highlight KCL error message, currently there is only one `DEFAULT`.

ColorOption (Enum): Fields that can be highlighted in the KCL error message.

KCLErrMsgTheme: An abstract class, which specifies the methods required by KCL error message theme classes.

KCLErrMsgThemeDefault: It is the default implementation class of KCLErrMsgTheme.

Default highlight：

    EWCODE: green
    ERROR: red
    WARNING: yellow
    FILE_NAME: blue
    LINE_COLUMN: cyan
    SYNTAX: dark yellow
    COMPLIE: dark green
    RUNTIME: dark blue
    MARK: {
        1: red
        2: purple
    },

:copyright: Copyright 2020 The KCL Authors. All rights reserved.
"""
from enum import Enum, unique
from kclvm.internal.util import PreCheck, CheckRules
from abc import ABCMeta, abstractmethod


@unique
class ThemeId(Enum):
    DEFAULT = 0
    OTHER = 1


@unique
class ColorOption(Enum):
    EWCODE = 0
    ERROR = 1
    WARNING = 2
    FILE_NAME = 3
    LINE_COLUMN = 4
    MARK = 5
    SYNTAX = 6
    COMPLIE = 7
    RUNTIME = 8


class KCLErrMsgTheme(metaclass=ABCMeta):
    @abstractmethod
    def color_mark(self, mark: str, mark_level: int = 1):
        pass

    @abstractmethod
    def color(self, content: str, color_option: ColorOption):
        pass


class KCLErrMsgThemeDefault(KCLErrMsgTheme):
    def __init__(self):
        self.KCL_THEME: dict = {
            ColorOption.EWCODE: "\033[0;92m{}\033[0m",  # green
            ColorOption.ERROR: "\033[0;91m{}\033[0m",  # red
            ColorOption.WARNING: "\033[0;93m{}\033[0m",  # yellow
            ColorOption.FILE_NAME: "\033[0;94m{}\033[0m",  # blue
            ColorOption.LINE_COLUMN: "\033[0;96m{}\033[0m",  # cyan
            ColorOption.SYNTAX: "\033[0;33m{}\033[0m",  # dark yellow
            ColorOption.COMPLIE: "\033[0;32m{}\033[0m",  # dark green
            ColorOption.RUNTIME: "\033[0;34m{}\033[0m",  # dark blue
            ColorOption.MARK: {
                1: "\033[0;31m{}\033[0m",  # red
                2: "\033[0;35m{}\033[0m",  # purple
            },
        }

    @PreCheck((lambda v: CheckRules.check_type_not_none(v, str)), "mark")
    @PreCheck((lambda v: CheckRules.check_type_not_none(v, int)), "mark_level")
    @PreCheck((lambda v: CheckRules.check_int_range_allow_none(v, 1, 3)), "mark_level")
    def color_mark(self, mark: str, mark_level: int = 1):
        return self.KCL_THEME[ColorOption.MARK][mark_level].format(mark)

    @PreCheck((lambda v: CheckRules.check_type_not_none(v, str)), "content")
    @PreCheck(
        (lambda v: CheckRules.check_type_not_none(v, ColorOption)), "color_option"
    )
    def color(self, content: str, color_option: ColorOption):
        return self.KCL_THEME[color_option].format(content)
