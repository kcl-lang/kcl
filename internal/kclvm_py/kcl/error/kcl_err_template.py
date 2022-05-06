"""
The `kcl_err_template` file mainly contains some string templates to organize the structure of the KCL error message.

The KCL error message comes from kcl_err_msg.py

KCLErrMsgTemplate: An abstract class, which specifies the methods required by KCL error message template classes.

KCLErrMsgTemplateDefault: The default KCL template class implementation.

Provide four file message templates in KCLErrMsgTemplateDefault:

    get_hint_msg_common:
----------------------------------------------------------------------------
        KCL Compile Error[E2L23] : A complie error occurs during compiling
        ---> File /main.k:2:14
        2 |    name: str
                     14 ^  -> Failure
        An error occurs
----------------------------------------------------------------------------

    get_hint_msg_tiny:
----------------------------------------------------------------------------
        KCL Compile Error[E2L23] : A compile error occurs during compiling
        An error occurs
----------------------------------------------------------------------------

    get_hint_msg_summary:
----------------------------------------------------------------------------
        KCL Compile Error[E2L23] : A complie error occurs during compiling
        ---> File /main.k:2
        2 |    name: str -> Failure
        An error occurs
----------------------------------------------------------------------------

    get_hint_msg_detail:
----------------------------------------------------------------------------
        KCL Compile Error[E2L23] : A complie error occurs during compiling
        ---> File /main.k:2:5
        2 |    name: str
             5 ^^^^^^^^^  -> Failure
        An error occurs
----------------------------------------------------------------------------

Provide one error message templates in KCLErrMsgTemplateDefault:

    err_msg_template:
----------------------------------------------------------------------------
    KCL Compile Error[E2L23] : A compile error occurs during compiling
----------------------------------------------------------------------------

color_txt_err_msg(), color_txt_err_file_msg() in KCLErrMsgTemplateDefault
will call methods provided in kcl_err_template.py to highlight some fields
in the kcl error message

:copyright: Copyright 2020 The KCL Authors. All rights reserved.
"""
import os
import typing
import threading

from enum import unique, Enum
from pathlib import PosixPath
from string import Template
from abc import ABCMeta, abstractmethod
from kclvm.kcl.error.kcl_err_theme import (
    ColorOption,
    KCLErrMsgTheme,
    KCLErrMsgThemeDefault,
)
from kclvm.kcl.error.kcl_err_msg import KCL_ERR_MSG, ErrEwcode
from kclvm.internal.util import PreCheck, CheckRules, PostCheck, check_utils

DEFAULT_MSG_2 = "Failure"


@unique
class ErrLevel(Enum):
    SERIOUS = 1
    ORDINARY = 2


@unique
class MsgId(Enum):
    ERR_TYPE = 1
    ERR_CATE = 2
    EWCODE = 3
    MSG_1 = 4
    MSG_2 = 5

    FILENAME = 8
    SRC_CODE_LINE = 9
    LINE_NO = 10
    COL_NO = 11
    ERR_ARG = 12
    MARK = 13


@unique
class ErrMsgFmt(Enum):
    COLOR_TXT = 1
    NO_COLOR_TXT = 2


class FileCache:
    """File cache to store the filename and code mapping"""

    _file_cache: typing.Dict[str, str] = {}
    _lock = threading.RLock()

    @staticmethod
    def clear():
        FileCache._file_cache.clear()

    @staticmethod
    def put(file: str, code: str):
        FileCache._lock.acquire()
        FileCache._file_cache[file] = code
        FileCache._lock.release()

    @staticmethod
    def get(file: str) -> str:
        return FileCache._file_cache.get(file, "")


UP_ARROW = "^"
WAVY = "~"

MARK_SYMBOL = {ErrLevel.SERIOUS: UP_ARROW, ErrLevel.ORDINARY: WAVY}

TAB = "    "
COLON = ":"
COMMA = ", "
WHITE_SPACE = " "
SPARATOER = " |"
NEW_LINE = "\n"
HINT_MSG = "{}"
FILE_PREFIX = "---> File "
MSG_PREFIX = "KCL "


class KCLErrMsgTemplate(metaclass=ABCMeta):
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, ErrMsgFmt)), "fmt")
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, KCLErrMsgTheme)), "theme")
    def __init__(
        self,
        fmt: ErrMsgFmt = ErrMsgFmt.NO_COLOR_TXT,
        theme: KCLErrMsgTheme = KCLErrMsgThemeDefault(),
    ):
        self.fmt = fmt
        self.theme = theme

    @abstractmethod
    def get_hint_msg_common(
        self,
        filename: str,
        line_no: int,
        col_no: int,
        src_code_line: str,
        indent_count: int = 0,
        mark: str = MARK_SYMBOL[ErrLevel.SERIOUS],
    ) -> str:
        pass

    @abstractmethod
    def get_hint_msg_tiny(self, filename: str, indent_count: int = 0) -> str:
        pass

    @abstractmethod
    def get_hint_msg_summary(
        self, filename: str, line_no: int, src_code_line: str, indent_count: int = 0
    ) -> str:
        pass

    @abstractmethod
    def get_hint_msg_detail(
        self,
        filename: str,
        line_no: int,
        col_no: int,
        end_col_no: int,
        src_code_line: str,
        indent_count: int = 0,
        mark: str = MARK_SYMBOL[ErrLevel.SERIOUS],
    ) -> str:
        pass

    @abstractmethod
    def err_msg_template(
        self, err_type: str, err_cate: str, ewcode_fmt: str, msg_1: str
    ):
        pass

    @abstractmethod
    def color_txt_err_file_msg(self, mark_level: ErrLevel):
        pass

    @abstractmethod
    def color_txt_err_msg(self, err_type: str, err_cate: str):
        pass


class KCLErrMsgTemplateDefault(KCLErrMsgTemplate):
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, ErrMsgFmt)), "fmt")
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, KCLErrMsgTheme)), "theme")
    def __init__(
        self,
        fmt: ErrMsgFmt = ErrMsgFmt.NO_COLOR_TXT,
        theme: KCLErrMsgTheme = KCLErrMsgThemeDefault(),
    ):
        super().__init__(fmt, theme)
        self.fmt = fmt
        self.theme = theme

        self.ErrMsgLables = {
            MsgId.ERR_TYPE: "${err_type}",
            MsgId.ERR_CATE: "${err_cate}",
            MsgId.EWCODE: "${ewcode}",
            MsgId.MSG_1: "${msg_1}",
            MsgId.MSG_2: "${msg_2}",
            MsgId.FILENAME: "${filename}",
            MsgId.SRC_CODE_LINE: "${src_code_line}",
            MsgId.LINE_NO: "${line_no}",
            MsgId.COL_NO: "${col_no}",
            MsgId.ERR_ARG: "${err_arg}",
            MsgId.MARK: "${mark}",
        }
        self.FILENAME = self.ErrMsgLables[MsgId.FILENAME]
        self.LINE_NO = self.ErrMsgLables[MsgId.LINE_NO]
        self.COL_NO = self.ErrMsgLables[MsgId.COL_NO]
        self.MARK = self.ErrMsgLables[MsgId.MARK]

        self.EWCODE = self.ErrMsgLables[MsgId.EWCODE]
        self.ERR_TYPE = self.ErrMsgLables[MsgId.ERR_TYPE]
        self.ERR_CATE = self.ErrMsgLables[MsgId.ERR_CATE]
        self.MSG_1 = self.ErrMsgLables[MsgId.MSG_1]
        self.MSG_2 = self.ErrMsgLables[MsgId.MSG_2]

        self.SRC_CODE_LINE = self.ErrMsgLables[MsgId.SRC_CODE_LINE]
        self.ERR_ARG = self.ErrMsgLables[MsgId.ERR_ARG]

    def clean_color(self):
        self.FILENAME = self.ErrMsgLables[MsgId.FILENAME]
        self.LINE_NO = self.ErrMsgLables[MsgId.LINE_NO]
        self.COL_NO = self.ErrMsgLables[MsgId.COL_NO]
        self.MARK = self.ErrMsgLables[MsgId.MARK]

        self.EWCODE = self.ErrMsgLables[MsgId.EWCODE]
        self.ERR_TYPE = self.ErrMsgLables[MsgId.ERR_TYPE]
        self.ERR_CATE = self.ErrMsgLables[MsgId.ERR_CATE]

    @PreCheck((lambda v: CheckRules.check_type_not_none(v, str)), "filename")
    @PreCheck((lambda v: CheckRules.check_type_not_none(v, int)), "line_no")
    @PreCheck((lambda v: CheckRules.check_type_not_none(v, int)), "col_no")
    @PreCheck((lambda v: CheckRules.check_type_not_none(v, str)), "src_code_line")
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, int)), "indent_count")
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "mark")
    @PostCheck(lambda v: CheckRules.check_type_not_none(v, str))
    def get_hint_msg_common(
        self,
        filename: str,
        line_no: int,
        col_no: int,
        src_code_line: str,
        indent_count: int = 0,
        mark: str = MARK_SYMBOL[ErrLevel.SERIOUS],
    ) -> str:
        HINT_MSG_COMMON = (
            NEW_LINE
            + TAB * indent_count
            + FILE_PREFIX
            + self.FILENAME
            + COLON
            + self.LINE_NO
            + COLON
            + self.COL_NO
            + NEW_LINE
            + TAB * indent_count
            + self.LINE_NO
            + SPARATOER
            + self.SRC_CODE_LINE
            + NEW_LINE
            + TAB * indent_count
            + (self.COL_NO + WHITE_SPACE).rjust(
                len(str(line_no)) + 1 + col_no + len(self.COL_NO) - len(str(col_no)),
                WHITE_SPACE,
            )
            + self.MARK
            + WHITE_SPACE
        )
        hint_msg_args = {
            "filename": filename,
            "line_no": line_no,
            "col_no": col_no,
            "src_code_line": src_code_line,
            "mark": mark,
        }
        return Template(HINT_MSG_COMMON).substitute(hint_msg_args)

    @PreCheck((lambda v: CheckRules.check_type_not_none(v, str)), "filename")
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, int)), "indent_count")
    @PostCheck(lambda v: CheckRules.check_type_not_none(v, str))
    def get_hint_msg_tiny(self, filename: str, indent_count: int = 0) -> str:
        HINT_MSG_TINY = NEW_LINE + TAB * indent_count + FILE_PREFIX + self.FILENAME
        hint_msg_args = {"filename": filename}
        return Template(HINT_MSG_TINY).substitute(hint_msg_args)

    @PreCheck((lambda v: CheckRules.check_type_not_none(v, str)), "filename")
    @PreCheck((lambda v: CheckRules.check_type_not_none(v, int)), "line_no")
    @PreCheck((lambda v: CheckRules.check_type_not_none(v, str)), "src_code_line")
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, int)), "indent_count")
    @PostCheck(lambda v: CheckRules.check_type_not_none(v, str))
    def get_hint_msg_summary(
        self, filename: str, line_no: int, src_code_line: str, indent_count: int = 0
    ) -> str:
        HINT_MSG_SUMMARY = (
            NEW_LINE
            + TAB * indent_count
            + FILE_PREFIX
            + self.FILENAME
            + COLON
            + self.LINE_NO
            + NEW_LINE
            + TAB * indent_count
            + self.LINE_NO
            + SPARATOER
            + self.SRC_CODE_LINE
        )
        hint_msg_args = {
            "filename": filename,
            "line_no": line_no,
            "src_code_line": src_code_line,
        }
        return Template(HINT_MSG_SUMMARY).substitute(hint_msg_args)

    @PreCheck((lambda v: CheckRules.check_type_not_none(v, str)), "filename")
    @PreCheck((lambda v: CheckRules.check_type_not_none(v, int)), "line_no")
    @PreCheck((lambda v: CheckRules.check_type_not_none(v, int)), "col_no")
    @PreCheck((lambda v: CheckRules.check_type_not_none(v, int)), "end_col_no")
    @PreCheck((lambda v: CheckRules.check_type_not_none(v, str)), "src_code_line")
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, int)), "indent_count")
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "mark")
    @PostCheck(lambda v: CheckRules.check_type_not_none(v, str))
    def get_hint_msg_detail(
        self,
        filename: str,
        line_no: int,
        col_no: int,
        end_col_no: int,
        src_code_line: str,
        indent_count: int = 0,
        mark: str = MARK_SYMBOL[ErrLevel.SERIOUS],
    ) -> str:
        HINT_MSG_DETAIL = (
            NEW_LINE
            + TAB * indent_count
            + FILE_PREFIX
            + self.FILENAME
            + COLON
            + self.LINE_NO
            + COLON
            + self.COL_NO
            + NEW_LINE
            + TAB * indent_count
            + self.LINE_NO
            + SPARATOER
            + self.SRC_CODE_LINE
            + NEW_LINE
            + TAB * indent_count
            + (self.COL_NO + WHITE_SPACE).rjust(
                len(str(line_no)) + 1 + col_no + len(self.COL_NO) - len(str(col_no)),
                WHITE_SPACE,
            )
            + self.MARK * (end_col_no - col_no)
            + WHITE_SPACE
        )
        hint_msg_args = {
            "filename": filename,
            "line_no": line_no,
            "col_no": col_no,
            "src_code_line": src_code_line,
            "mark": mark,
        }
        return Template(HINT_MSG_DETAIL).substitute(hint_msg_args)

    @PreCheck((lambda v: CheckRules.check_type_not_none(v, str)), "err_type")
    @PreCheck((lambda v: CheckRules.check_type_not_none(v, str)), "err_cate")
    @PreCheck((lambda v: CheckRules.check_type_not_none(v, str)), "ewcode_fmt")
    @PreCheck((lambda v: CheckRules.check_type_not_none(v, str)), "msg_1")
    @PostCheck(lambda v: CheckRules.check_type_not_none(v, str))
    def err_msg_template(
        self, err_type: str, err_cate: str, ewcode_fmt: str, msg_1: str
    ):
        SIMPLE_TEMPLATE_DOC = (
            MSG_PREFIX
            + self.ERR_TYPE
            + WHITE_SPACE
            + self.ERR_CATE
            + self.EWCODE
            + WHITE_SPACE
            + COLON
            + WHITE_SPACE
            + self.MSG_1
        )
        simple_args = {
            "err_type": err_type,
            "err_cate": err_cate,
            "ewcode": ewcode_fmt,
            "msg_1": msg_1,
        }
        return Template(SIMPLE_TEMPLATE_DOC).substitute(simple_args)

    @PreCheck((lambda v: CheckRules.check_type_not_none(v, ErrLevel)), "mark_level")
    def color_txt_err_file_msg(self, mark_level: ErrLevel):
        self.MARK = self.theme.color_mark(self.MARK, mark_level.value)
        self.FILENAME = self.theme.color(self.FILENAME, ColorOption.FILE_NAME)
        self.LINE_NO = self.theme.color(self.LINE_NO, ColorOption.LINE_COLUMN)
        self.COL_NO = self.theme.color(self.COL_NO, ColorOption.LINE_COLUMN)

    @PreCheck((lambda v: CheckRules.check_type_not_none(v, str)), "err_type")
    @PreCheck((lambda v: CheckRules.check_type_not_none(v, str)), "err_cate")
    def color_txt_err_msg(self, err_type: str, err_cate: str):
        self.EWCODE = self.theme.color(self.EWCODE, ColorOption.EWCODE)
        self.ERR_TYPE = self.color_err_msg(slot=self.ERR_TYPE, msg=err_type)
        self.ERR_CATE = self.color_err_msg(slot=self.ERR_CATE, msg=err_cate)

    @PreCheck((lambda v: CheckRules.check_type_not_none(v, str)), "msg")
    @PreCheck((lambda v: CheckRules.check_type_not_none(v, str)), "slot")
    @PostCheck(lambda v: CheckRules.check_type_not_none(v, str))
    def color_err_msg(self, slot: str, msg: str):
        msg_theme_map: dict = {
            KCL_ERR_MSG.get_err_msg_by_errid(ErrEwcode.KCLError_Ew): ColorOption.ERROR,
            KCL_ERR_MSG.get_err_msg_by_errid(
                ErrEwcode.KCLWarning_Ew
            ): ColorOption.WARNING,
            KCL_ERR_MSG.get_err_msg_by_errid(
                ErrEwcode.KCLSyntaxException_Ew
            ): ColorOption.SYNTAX,
            KCL_ERR_MSG.get_err_msg_by_errid(
                ErrEwcode.KCLCompileException_Ew
            ): ColorOption.COMPLIE,
            KCL_ERR_MSG.get_err_msg_by_errid(
                ErrEwcode.KCLRuntimeException_Ew
            ): ColorOption.RUNTIME,
            KCL_ERR_MSG.get_err_msg_by_errid(
                ErrEwcode.KCLException_Ew
            ): ColorOption.ERROR,
        }
        try:
            return self.theme.color(slot, msg_theme_map[msg])
        except KeyError:
            check_utils.alert_internal_bug()


class ErrFileMsg:
    @PreCheck(
        (lambda v: CheckRules.check_type_allow_none(v, str, PosixPath)), "filename"
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, int)), "line_no")
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, int)), "col_no")
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, int)), "end_col_no")
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "src_code")
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, int)), "indent_count")
    def __init__(
        self,
        filename: typing.Union[str, PosixPath] = None,
        line_no: int = None,
        col_no: int = None,
        end_col_no: int = None,
        src_code: str = None,
        arg_msg: str = DEFAULT_MSG_2,
        indent_count: int = 0,
        err_level: ErrLevel = ErrLevel.SERIOUS,
    ):
        self.filename = filename
        self.line_no = line_no
        self.col_no = col_no
        self.end_col_no = end_col_no
        self.src_code = src_code
        self.err_level = err_level if err_level else ErrLevel.SERIOUS
        self.arg_msg = arg_msg if arg_msg else DEFAULT_MSG_2
        self.indent_count = indent_count


@PreCheck((lambda v: CheckRules.check_type_not_none(v, ErrFileMsg)), "file_msg")
@PostCheck(lambda v: CheckRules.check_type_not_none(v, str))
def get_src_code(file_msg: ErrFileMsg) -> str:
    """Produce formatted error hint messages"""
    src_line = ""
    if file_msg.filename:
        source_line: str = ""
        if file_msg.line_no:
            if file_msg.src_code:
                lines = file_msg.src_code.split("\n")
            else:
                lines = (
                    open(file_msg.filename, "r", encoding="utf8").read().split("\n")
                    if os.path.exists(file_msg.filename)
                    else FileCache.get(file_msg.filename).split("\n")
                )

            if 0 < file_msg.line_no <= len(lines):
                source_line = lines[file_msg.line_no - 1]
            return source_line
    elif file_msg.src_code and file_msg.line_no:
        lines = file_msg.src_code.split("\n")
        if 0 < file_msg.line_no <= len(lines):
            src_line = lines[file_msg.line_no - 1]
        return src_line
    return ""


@PreCheck((lambda v: CheckRules.check_type_not_none(v, ErrFileMsg)), "file_msg")
@PostCheck(lambda v: CheckRules.check_type_not_none(v, str))
def get_hint_msg(
    file_msg: ErrFileMsg, msg_tem: KCLErrMsgTemplate = KCLErrMsgTemplateDefault()
) -> str:
    if msg_tem is None:
        msg_tem = KCLErrMsgTemplateDefault()
    msg_tem.clean_color()
    if msg_tem.fmt == ErrMsgFmt.COLOR_TXT:
        msg_tem.color_txt_err_file_msg(file_msg.err_level)
    result = ""
    arg = " -> " + file_msg.arg_msg if file_msg.arg_msg else ""
    mark = MARK_SYMBOL[file_msg.err_level]
    if file_msg.filename:
        if file_msg.line_no:
            src_code = get_src_code(file_msg)
            if file_msg.col_no and file_msg.end_col_no:
                result = (
                    msg_tem.get_hint_msg_detail(
                        file_msg.filename,
                        file_msg.line_no,
                        file_msg.col_no,
                        file_msg.end_col_no,
                        src_code,
                        indent_count=file_msg.indent_count,
                        mark=mark,
                    )
                    + arg
                )
            elif file_msg.col_no and not file_msg.end_col_no:
                result = (
                    msg_tem.get_hint_msg_common(
                        file_msg.filename,
                        file_msg.line_no,
                        file_msg.col_no,
                        src_code,
                        indent_count=file_msg.indent_count,
                        mark=mark,
                    )
                    + arg
                )
            elif not file_msg.col_no and not file_msg.end_col_no:
                result = (
                    msg_tem.get_hint_msg_summary(
                        file_msg.filename,
                        file_msg.line_no,
                        src_code,
                        indent_count=file_msg.indent_count,
                    )
                    + arg
                )
        else:
            result = (
                msg_tem.get_hint_msg_tiny(
                    file_msg.filename, indent_count=file_msg.indent_count
                )
                + arg
            )

    return result


@PreCheck((lambda v: CheckRules.check_type_not_none(v, str)), "ewcode")
@PostCheck(lambda v: CheckRules.check_type_not_none(v, str))
def get_err_msg(
    ewcode: str, msg_tem: KCLErrMsgTemplate = KCLErrMsgTemplateDefault()
) -> str:
    if msg_tem is None:
        msg_tem = KCLErrMsgTemplateDefault()
    msg_tem.clean_color()
    err_type = KCL_ERR_MSG.get_err_type_msg_by_errid(ewcode)
    err_cate = KCL_ERR_MSG.get_err_cate_msg_by_errid(ewcode)
    ewcode_fmt = KCL_ERR_MSG.get_err_code_by_errid(ewcode)
    err_msg = KCL_ERR_MSG.get_err_msg_by_errid(ewcode)

    if msg_tem.fmt == ErrMsgFmt.COLOR_TXT:
        msg_tem.color_txt_err_msg(err_type, err_cate)
    return msg_tem.err_msg_template(err_type, err_cate, ewcode_fmt, err_msg)
