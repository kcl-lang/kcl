"""
The `kcl_error` file mainly contains all KCL exceptions.

KCLException is the top-level exception class, all KCL exceptions inherit from KCLException.

KCLException.err_info_stack :
A file information stack is built in KCLException to save file information when an exception occurs.

KCLException.gen_err_msg :
Method `gen_err_msg` generates KCL error message without highlighting
by calling the method provided in kcl_err_template.py

KCLException.show_msg_with_theme :
Method `show_msg_with_theme` generates KCL error message with highlighting
by calling the method provided in kcl_err_template.py

:note: At present, the KCL error message templates of the two methods `gen_err_msg` and `show_msg_with_theme`
are created by default arguments, and they are not synchronized.
If the template of one method is changed,
the template of the other method also needs to be replaced manually,
otherwise the message output by the two methods will be different.

:copyright: Copyright 2020 The KCL Authors. All rights reserved.
"""
import sys
import typing
from enum import Enum, unique

import kclvm.kcl.error.kcl_err_template as err_template

from kclvm.kcl.error.kcl_err_msg import KCL_ERR_MSG, ErrEwcode
from kclvm.internal.util import PreCheck, CheckRules, PostCheck


class KCLException(Exception):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "ewcode")
    @PreCheck(
        (
            lambda v: CheckRules.check_str_len_allow_none(
                v, len(KCL_ERR_MSG.get_defaule_ewcode())
            )
        ),
        "ewcode",
    )
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        ewcode: str = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLException_Ew if not ewcode else ewcode
        self.name = KCL_ERR_MSG.get_err_name_by_ewcode(self.ewcode)
        self.err_info_stack: typing.List[err_template.ErrFileMsg] = []
        if file_msgs:
            self.err_info_stack.extend(file_msgs)
        self.arg_msg = (
            arg_msg
            if arg_msg
            else KCL_ERR_MSG.get_defaule_arg_msg_by_errid(self.ewcode)
        )

    def __str__(self):
        return self.gen_err_msg()

    def show_msg_with_theme(
        self,
        color_template=err_template.KCLErrMsgTemplateDefault(
            fmt=err_template.ErrMsgFmt.COLOR_TXT
        ),
    ) -> str:
        if not color_template:
            color_template = err_template.KCLErrMsgTemplateDefault(
                fmt=err_template.ErrMsgFmt.COLOR_TXT
            )
        return self.gen_err_msg(msg_tem=color_template)

    def gen_err_msg(self, msg_tem=err_template.KCLErrMsgTemplateDefault()) -> str:
        result = ""
        result += err_template.get_err_msg(self.ewcode, msg_tem)
        count = 0
        for file in reversed(self.err_info_stack):
            if file and file.filename:
                file.indent_count = count
                result += err_template.get_hint_msg(file, msg_tem)
                count += 1
        return result + "\n" + self.arg_msg if self.arg_msg else result

    def no_err_msg(self) -> bool:
        return self.err_info_stack is None or len(self.err_info_stack) == 0

    def append_err_info(self, einfo: err_template.ErrFileMsg):
        self.err_info_stack.append(einfo)

    def pop_err_info(self) -> err_template.ErrFileMsg:
        return (
            self.err_info_stack.pop(-1)
            if len(self.err_info_stack) > 0
            else err_template.ErrFileMsg()
        )

    @property
    def filename(self):
        return self.err_info_stack[-1].filename if len(self.err_info_stack) > 0 else ""

    @property
    def lineno(self):
        return self.err_info_stack[-1].line_no if len(self.err_info_stack) > 0 else ""

    @property
    def colno(self):
        return self.err_info_stack[-1].col_no if len(self.err_info_stack) > 0 else ""


class KCLError(KCLException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLWarning(KCLException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLWarning_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLSyntaxException(KCLException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLSyntaxException_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLCompileException(KCLException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLCompileException_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLRuntimeException(KCLException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLRuntimeException_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLAttributeException(KCLException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLAttributeException_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLSchemaException(KCLException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLSchemaException_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLMixinException(KCLException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLMixinException_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLInheritException(KCLException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLInheritException_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLImportException(KCLException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLImportException_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLTypeException(KCLException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLTypeException_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLDecoratorException(KCLException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLDecoratorException_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLArgumentException(KCLException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLArgumentException_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLOverflowException(KCLException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLOverflowException_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLComplingException(KCLException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLComplingException_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLRunningException(KCLException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLRunningException_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLDeprecatedException(KCLException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLDeprecatedException_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLDocException(KCLException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLDocException_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLImmutableException(KCLException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLImmutableException_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class InvalidSyntaxError(KCLError, KCLSyntaxException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.InvalidSyntax_Ew
        self.accepts_lark = []
        self.accepts_msg = []
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLTabError(KCLError, KCLSyntaxException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLTabError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLIndentationError(KCLError, KCLSyntaxException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLIndentationError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class CannotFindModule(KCLError, KCLCompileException, KCLImportException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.CannotFindModule_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class FailedLoadModule(KCLError, KCLCompileException, KCLImportException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.FailedLoadModule_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class RecursiveLoad(KCLError, KCLCompileException, KCLImportException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.RecursiveLoad_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class FloatOverflow(KCLError, KCLRuntimeException, KCLOverflowException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.FloatOverflow_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class FloatUnderflow(KCLWarning, KCLCompileException, KCLOverflowException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.FloatUnderflow_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class IntOverflow(KCLError, KCLRuntimeException, KCLOverflowException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.IntOverflow_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class InvalidDocstring(KCLWarning, KCLCompileException, KCLDocException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.InvalidDocstring_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class DeprecatedError(KCLError, KCLRuntimeException, KCLDeprecatedException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.Deprecated_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class DeprecatedWarning(KCLWarning, KCLDeprecatedException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.DeprecatedWarning_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class UnKnownDecoratorError(KCLError, KCLCompileException, KCLDecoratorException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.UnKnownDecorator_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class InvalidDecoratorTargetError(KCLError, KCLCompileException, KCLDecoratorException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.InvalidDecoratorTarget_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class MixinNamingError(KCLError, KCLCompileException, KCLMixinException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.MixinNamingError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class MixinStructureIllegal(KCLError, KCLCompileException, KCLMixinException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.MixinStructureIllegal_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class SchemaCheckFailure(KCLError, KCLRuntimeException, KCLSchemaException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.SchemaCheckFailure_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class CannotAddMembersComplieError(KCLError, KCLCompileException, KCLSchemaException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.CannotAddMembersComplieError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class CannotAddMembersRuntimeError(KCLError, KCLRuntimeException, KCLSchemaException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.CannotAddMembersRuntimeError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class IndexSignatureError(KCLError, KCLCompileException, KCLSchemaException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.IndexSignatureError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class TypeRuntimeError(KCLError, KCLRuntimeException, KCLTypeException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.TypeRuntimeError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class TypeComplieError(KCLError, KCLCompileException, KCLTypeException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.TypeComplieError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class CompileError(KCLError, KCLCompileException, KCLComplingException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.CompileError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class SelectorError(KCLError, KCLCompileException, KCLComplingException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.SelectorError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLNameError(KCLError, KCLCompileException, KCLComplingException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLNameError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLValueError(KCLError, KCLCompileException, KCLComplingException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLValueError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLKeyError(KCLError, KCLCompileException, KCLComplingException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLKeyError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class UniqueKeyError(KCLError, KCLCompileException, KCLComplingException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.UniqueKeyError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLAttributeComplieError(KCLError, KCLCompileException, KCLAttributeException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLAttributeComplieError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLAttributeRuntimeError(KCLError, KCLRuntimeException, KCLAttributeException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLAttributeRuntimeError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class IllegalAttributeError(KCLError, KCLCompileException, KCLAttributeException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.IllegalAttributeError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class MultiInheritError(KCLError, KCLCompileException, KCLInheritException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.MultiInheritError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class CycleInheritError(KCLError, KCLCompileException, KCLInheritException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.CycleInheritError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class IllegalInheritError(KCLError, KCLCompileException, KCLInheritException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.IllegalInheritError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class IllegalArgumentRuntimeError(KCLError, KCLRuntimeException, KCLArgumentException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.IllegalArgumentRuntimeError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class IllegalArgumentComplieError(KCLError, KCLCompileException, KCLArgumentException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.IllegalArgumentComplieError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class IllegalArgumentSyntaxError(KCLError, KCLSyntaxException, KCLArgumentException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.IllegalArgumentSyntaxError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class EvaluationError(KCLError, KCLRuntimeException, KCLRunningException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.EvaluationError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class InvalidFormatSpec(KCLError, KCLRuntimeException, KCLRunningException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.InvalidFormatSpec_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLAssertionError(KCLError, KCLRuntimeException, KCLRunningException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLAssertionError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class ImmutableRuntimeError(KCLError, KCLCompileException, KCLImmutableException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.ImmutableRuntimeError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class ImmutableCompileError(KCLError, KCLCompileException, KCLImmutableException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.ImmutableCompileError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class KCLRecursionError(KCLError, KCLRuntimeException, KCLRunningException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.KCLRecursionError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


class PlanError(KCLError, KCLRuntimeException, KCLRunningException):
    @PreCheck(
        (
            lambda v: CheckRules.check_list_item_type_allow_none(
                v, err_template.ErrFileMsg
            )
        ),
        "file_msgs",
    )
    @PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
    def __init__(
        self,
        file_msgs: typing.List[err_template.ErrFileMsg] = None,
        arg_msg: str = None,
    ):
        self.ewcode = ErrEwcode.PlanError_Ew
        KCLException.__init__(
            self, file_msgs=file_msgs, ewcode=self.ewcode, arg_msg=arg_msg
        )


@unique
class ErrType(Enum):
    InvalidSyntax_TYPE = (0,)
    TabError_TYPE = (1,)
    IndentationError_TYPE = (2,)
    CannotFindModule_TYPE = (3,)
    FailedLoadModule_TYPE = (4,)
    CompileError_TYPE = (5,)
    EvaluationError_TYPE = (6,)
    RecursiveLoad_TYPE = (7,)
    FloatOverflow_TYPE = (8,)
    FloatUnderflow_TYPE = (9,)
    IntOverflow_TYPE = (10,)
    InvalidDocstring_TYPE = (11,)
    Deprecated_TYPE = (12,)
    UnKnownDecorator_TYPE = (13,)
    InvalidDecoratorTarget_TYPE = (14,)
    InvalidFormatSpec_TYPE = (15,)
    SelectorError_TYPE = (16,)
    SchemaCheckFailure_TYPE = (17,)
    MixinNamingError_TYPE = (18,)
    MixinStructureIllegal_TYPE = (19,)

    IndexSignatureError_TYPE = (20,)
    TypeError_Runtime_TYPE = (21,)
    TypeError_Compile_TYPE = (22,)
    NameError_TYPE = (23,)
    ValueError_TYPE = (24,)
    KeyError_TYPE = (25,)
    UniqueKeyError_TYPE = (26,)
    AttributeError_TYPE = (27,)
    AttributeError_Runtime_TYPE = (28,)
    AssertionError_TYPE = (29,)
    ImmutableCompileError_TYPE = (30,)
    ImmutableRuntimeError_TYPE = (31,)
    MultiInheritError_TYPE = (32,)
    CycleInheritError_TYPE = (33,)
    IllegalInheritError_TYPE = (34,)
    IllegalAttributeError_TYPE = (35,)
    IllegalArgumentError_TYPE = (36,)
    IllegalArgumentError_Complie_TYPE = (37,)
    IllegalArgumentError_Syntax_TYPE = (38,)
    RecursionError_TYPE = (39,)
    PlanError_TYPE = (40,)
    Deprecated_Warning_TYPE = (41,)
    CannotAddMembers_TYPE = (42,)
    CannotAddMembers_Runtime_TYPE = (43,)


ERR_TYPE_EWCODE_MAP = {
    ErrType.InvalidSyntax_TYPE: InvalidSyntaxError,
    ErrType.TabError_TYPE: KCLTabError,
    ErrType.IndentationError_TYPE: KCLIndentationError,
    ErrType.CannotFindModule_TYPE: CannotFindModule,
    ErrType.FailedLoadModule_TYPE: FailedLoadModule,
    ErrType.CompileError_TYPE: CompileError,
    ErrType.EvaluationError_TYPE: EvaluationError,
    ErrType.RecursiveLoad_TYPE: RecursiveLoad,
    ErrType.FloatOverflow_TYPE: FloatOverflow,
    ErrType.FloatUnderflow_TYPE: FloatUnderflow,
    ErrType.IntOverflow_TYPE: IntOverflow,
    ErrType.InvalidDocstring_TYPE: InvalidDocstring,
    ErrType.Deprecated_TYPE: DeprecatedError,
    ErrType.Deprecated_Warning_TYPE: DeprecatedWarning,
    ErrType.UnKnownDecorator_TYPE: UnKnownDecoratorError,
    ErrType.InvalidDecoratorTarget_TYPE: InvalidDecoratorTargetError,
    ErrType.InvalidFormatSpec_TYPE: InvalidFormatSpec,
    ErrType.SelectorError_TYPE: SelectorError,
    ErrType.SchemaCheckFailure_TYPE: SchemaCheckFailure,
    ErrType.MixinNamingError_TYPE: MixinNamingError,
    ErrType.MixinStructureIllegal_TYPE: MixinStructureIllegal,
    ErrType.CannotAddMembers_TYPE: CannotAddMembersComplieError,
    ErrType.CannotAddMembers_Runtime_TYPE: CannotAddMembersRuntimeError,
    ErrType.IndexSignatureError_TYPE: IndexSignatureError,
    ErrType.TypeError_Runtime_TYPE: TypeRuntimeError,
    ErrType.TypeError_Compile_TYPE: TypeComplieError,
    ErrType.NameError_TYPE: KCLNameError,
    ErrType.ValueError_TYPE: KCLValueError,
    ErrType.KeyError_TYPE: KCLKeyError,
    ErrType.UniqueKeyError_TYPE: UniqueKeyError,
    ErrType.AttributeError_TYPE: KCLAttributeComplieError,
    ErrType.AttributeError_Runtime_TYPE: KCLAttributeRuntimeError,
    ErrType.AssertionError_TYPE: KCLAssertionError,
    ErrType.ImmutableCompileError_TYPE: ImmutableCompileError,
    ErrType.ImmutableRuntimeError_TYPE: ImmutableRuntimeError,
    ErrType.MultiInheritError_TYPE: MultiInheritError,
    ErrType.CycleInheritError_TYPE: CycleInheritError,
    ErrType.IllegalInheritError_TYPE: IllegalInheritError,
    ErrType.IllegalAttributeError_TYPE: IllegalAttributeError,
    ErrType.IllegalArgumentError_TYPE: IllegalArgumentRuntimeError,
    ErrType.IllegalArgumentError_Complie_TYPE: IllegalArgumentComplieError,
    ErrType.IllegalArgumentError_Syntax_TYPE: IllegalArgumentSyntaxError,
    ErrType.RecursionError_TYPE: KCLRecursionError,
    ErrType.PlanError_TYPE: PlanError,
}


@PreCheck(
    (lambda v: CheckRules.check_list_item_type_allow_none(v, err_template.ErrFileMsg)),
    "file_msgs",
)
@PreCheck((lambda v: CheckRules.check_type_not_none(v, ErrType)), "err_type")
@PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
@PostCheck(lambda result: CheckRules.check_type_not_none(result, KCLException))
def get_exception(
    err_type: ErrType = None,
    file_msgs: typing.List[err_template.ErrFileMsg] = None,
    arg_msg: str = None,
):
    return ERR_TYPE_EWCODE_MAP[err_type](file_msgs=file_msgs, arg_msg=arg_msg)


@PreCheck(
    (lambda v: CheckRules.check_list_item_type_allow_none(v, err_template.ErrFileMsg)),
    "file_msgs",
)
@PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
@PreCheck((lambda v: CheckRules.check_type_not_none(v, ErrType)), "err_type")
def report_exception(
    err_type: ErrType,
    file_msgs: typing.List[err_template.ErrFileMsg] = None,
    arg_msg: str = None,
):
    raise get_exception(err_type, file_msgs, arg_msg)


@PreCheck(
    (lambda v: CheckRules.check_list_item_type_allow_none(v, err_template.ErrFileMsg)),
    "file_msgs",
)
@PreCheck((lambda v: CheckRules.check_type_allow_none(v, str)), "arg_msg")
@PreCheck((lambda v: CheckRules.check_type_not_none(v, ErrType)), "err_type")
def report_warning(
    err_type: ErrType,
    file_msgs: typing.List[err_template.ErrFileMsg] = None,
    arg_msg: str = None,
):
    print_kcl_warning_message(
        get_exception(err_type, file_msgs, arg_msg), file=sys.stderr
    )


@PreCheck((lambda v: CheckRules.check_type_not_none(v, KCLException)), "err")
def print_kcl_error_message(err: KCLException, file=sys.stderr):
    err_msg = err.show_msg_with_theme() if file.isatty() else (str(err))
    print(err_msg, file=file)


@PreCheck((lambda v: CheckRules.check_type_not_none(v, KCLWarning)), "err")
def print_kcl_warning_message(err: KCLWarning, file=sys.stderr):
    err_msg = err.show_msg_with_theme() if file.isatty() else (str(err))
    print(err_msg, file=file)


@PreCheck((lambda v: CheckRules.check_type_not_none(v, Exception)), "err")
def print_common_error_message(err: Exception, file=sys.stderr):
    print("Error: {0}".format(err), file=file)


@PreCheck((lambda v: CheckRules.check_type_allow_none(v, Exception)), "err")
def print_internal_error_message(err: Exception = None, file=sys.stderr):
    if err:
        print(err, file=file)
    print("Internal Error! Please report a bug to us.", file=file)
