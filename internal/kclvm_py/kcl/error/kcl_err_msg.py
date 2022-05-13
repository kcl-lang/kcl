"""
The `kcl_err_msg` file mainly contains constants used in KCL error messages.

INT_OVER_FLOW_MSG: String constants ending with "_MSG" are the recommended string template when raising KCL exceptions.

ErrEwcode: All ewcodes for KCL exception.

ErrMsgContent_EN: A dict for kcl exception message,
                    <key: value> = <ewcode: message>

ErrArgMsgDefault_EN: A dict for default arguments provided when a kcl exception is raised,
                        <key: value> = <ewcode: arg>

ErrName_EN: A dict for kcl exception names,
                <key: value> = <ewcode: exception_name>

KCLErrMsgManager: A singleton class that encapsulates some methods of obtaining KCL exception information

:copyright: Copyright 2020 The KCL Authors. All rights reserved.
"""
import sys
from abc import ABCMeta, abstractmethod
from kclvm.internal.util import PreCheck, CheckRules, PostCheck
from kclvm.internal.util.check_utils import PostSimpleExprCheck

ERROR = "E"
WARNING = "W"

SYNTAX = "1"
COMPLIER = "2"
RUNTIME = "3"

ATTRIBUTE = "A"
SCHEMA = "B"
MIXIN = "C"
INHERIT = "D"
IMPORT = "F"
TYPE = "G"
DECORATOR = "H"
ARGUMENT = "I"
OVERFLOW = "K"
COMPLING = "L"
RUNNING = "M"
DEPRECATED = "N"
DOC = "P"
IMMUTABLE = "Q"

INT_OVER_FLOW_MSG = "{}: A {} bit integer overflow"
FLOAT_OVER_FLOW_MSG = "{}: A {}-bit floating point number overflow"
FLOAT_UNDER_FLOW_MSG = "{}: A {}-bit floating point number underflow"
DEPRECATED_WARNING_MSG = "{} was deprecated {}"

SCHEMA_CHECK_FILE_MSG_ERR = "Instance check failed"
SCHEMA_CHECK_FILE_MSG_COND = "Check failed on the condition"

UNIQUE_KEY_MSG = "Variable name '{}' must be unique in package context"
CANNOT_ADD_MEMBERS_MSG = "{}: No such member in the schema '{}'"
RECURSIVE_LOADING_MODULE_MSG = "In module {}, recursively loading modules: {}"
CANNOT_FIND_MODULE_MSG = "Cannot find the module {} from {}"
MULTI_INHERIT_MSG = "Multiple inheritance of {} is prohibited"
INDENTATION_ERROR_MSG = "Unindent {} does not match any outer indentation level"
UNKNOWN_DECORATOR_MSG = "UnKnown decorator {}"
INVALID_DECORATOR_TARGET_MSG = "Invalid decorator target {}"
NAME_ERROR_MSG = "Name error {}"
INVALID_FORMAT_SPEC_MSG = "{} is invalid format spec"


class ErrEwcode:
    KCLException_Ew = "00000"  # 00000
    KCLError_Ew = ERROR + "0000"  # E0000
    KCLWarning_Ew = WARNING + "0000"  # W0000

    KCLSyntaxException_Ew = "0" + SYNTAX + "000"  # 01000
    KCLCompileException_Ew = "0" + COMPLIER + "000"  # 02000
    KCLRuntimeException_Ew = "0" + RUNTIME + "000"  # 03000

    KCLAttributeException_Ew = "00" + ATTRIBUTE + "00"  # 00A00
    KCLSchemaException_Ew = "00" + SCHEMA + "00"  # 00B00
    KCLMixinException_Ew = "00" + MIXIN + "00"  # 00C00
    KCLInheritException_Ew = "00" + INHERIT + "00"  # 00D00
    KCLImportException_Ew = "00" + IMPORT + "00"  # 00F00
    KCLTypeException_Ew = "00" + TYPE + "00"  # 00G00
    KCLDecoratorException_Ew = "00" + DECORATOR + "00"  # 00H00
    KCLArgumentException_Ew = "00" + ARGUMENT + "00"  # 00I00
    KCLOverflowException_Ew = "00" + OVERFLOW + "00"  # 00K00
    KCLComplingException_Ew = "00" + COMPLING + "00"  # 00L00
    KCLRunningException_Ew = "00" + RUNNING + "00"  # 00M00
    KCLDeprecatedException_Ew = "00" + DEPRECATED + "00"  # 00N00
    KCLDocException_Ew = "00" + DOC + "00"  # 00P00
    KCLImmutableException_Ew = "00" + IMMUTABLE + "00"  # 00Q00

    InvalidSyntax_Ew = ERROR + SYNTAX + "001"  # E1001
    KCLTabError_Ew = ERROR + SYNTAX + "002"  # E1002
    KCLIndentationError_Ew = ERROR + SYNTAX + "003"  # E1003

    CannotFindModule_Ew = ERROR + COMPLIER + IMPORT + "04"  # E2F04
    FailedLoadModule_Ew = ERROR + COMPLIER + IMPORT + "05"  # E2F05
    RecursiveLoad_Ew = ERROR + COMPLIER + IMPORT + "06"  # E3F06

    FloatOverflow_Ew = ERROR + RUNTIME + OVERFLOW + "07"  # E3K04
    FloatUnderflow_Ew = WARNING + COMPLIER + OVERFLOW + "08"  # W2K08
    IntOverflow_Ew = ERROR + RUNTIME + OVERFLOW + "09"  # E3K09

    InvalidDocstring_Ew = WARNING + COMPLIER + DOC + "10"  # W2P10

    Deprecated_Ew = ERROR + RUNTIME + DEPRECATED + "11"  # E3N11
    DeprecatedWarning_Ew = WARNING + COMPLIER + DEPRECATED + "12"  # W2N12

    UnKnownDecorator_Ew = ERROR + COMPLIER + DECORATOR + "13"  # E2H13
    InvalidDecoratorTarget_Ew = ERROR + COMPLIER + DECORATOR + "14"  # E2H14

    MixinNamingError_Ew = ERROR + COMPLIER + MIXIN + "15"  # E2C15
    MixinStructureIllegal_Ew = ERROR + COMPLIER + MIXIN + "16"  # E2C16

    SchemaCheckFailure_Ew = ERROR + RUNTIME + SCHEMA + "17"  # E3B17
    CannotAddMembersComplieError_Ew = ERROR + COMPLIER + SCHEMA + "18"  # E2B17
    CannotAddMembersRuntimeError_Ew = ERROR + RUNTIME + SCHEMA + "19"  # E3B19
    IndexSignatureError_Ew = ERROR + COMPLIER + SCHEMA + "20"  # E2B20

    TypeRuntimeError_Ew = ERROR + RUNTIME + TYPE + "21"  # E3G21
    TypeComplieError_Ew = ERROR + COMPLIER + TYPE + "22"  # E2G22

    CompileError_Ew = ERROR + COMPLIER + COMPLING + "23"  # E2L23
    SelectorError_Ew = ERROR + COMPLIER + COMPLING + "24"  # E2L24
    KCLNameError_Ew = ERROR + COMPLIER + COMPLING + "25"  # E2L25
    KCLValueError_Ew = ERROR + COMPLIER + COMPLING + "26"  # E2L26
    KCLKeyError_Ew = ERROR + COMPLIER + COMPLING + "27"  # E2L27
    UniqueKeyError_Ew = ERROR + COMPLIER + COMPLING + "28"  # E2L28

    KCLAttributeComplieError_Ew = ERROR + COMPLIER + ATTRIBUTE + "29"  # E2A29
    KCLAttributeRuntimeError_Ew = ERROR + RUNTIME + ATTRIBUTE + "30"  # E3A30
    IllegalAttributeError_Ew = ERROR + COMPLIER + ATTRIBUTE + "31"  # E2A31

    MultiInheritError_Ew = ERROR + COMPLIER + INHERIT + "32"  # E2D32
    CycleInheritError_Ew = ERROR + COMPLIER + INHERIT + "33"  # E2D33
    IllegalInheritError_Ew = ERROR + COMPLIER + INHERIT + "34"  # E2D34

    IllegalArgumentRuntimeError_Ew = ERROR + RUNTIME + ARGUMENT + "35"  # E3I35
    IllegalArgumentComplieError_Ew = ERROR + COMPLIER + ARGUMENT + "36"  # E2I36
    IllegalArgumentSyntaxError_Ew = ERROR + SYNTAX + ARGUMENT + "37"  # E1I37

    EvaluationError_Ew = ERROR + RUNTIME + RUNNING + "38"  # E3M38
    InvalidFormatSpec_Ew = ERROR + RUNTIME + RUNNING + "39"  # E3M39
    KCLAssertionError_Ew = ERROR + RUNTIME + RUNNING + "40"  # E3M40

    ImmutableCompileError_Ew = ERROR + COMPLIER + COMPLING + "41"  # E3L41
    KCLRecursionError_Ew = ERROR + RUNTIME + RUNNING + "42"  # E3M42
    PlanError_Ew = ERROR + RUNTIME + RUNNING + "43"  # E3M43
    ImmutableRuntimeError_Ew = ERROR + RUNTIME + RUNNING + "44"  # E3M44

    @staticmethod
    def contains(ewcode: str):
        return ewcode in ErrEwcode.__dict__.values()


ErrMsgContent_EN: dict = {
    ErrEwcode.KCLException_Ew: "Exception",
    ErrEwcode.KCLError_Ew: "Error",
    ErrEwcode.KCLWarning_Ew: "Warning",
    ErrEwcode.KCLSyntaxException_Ew: "Syntax",
    ErrEwcode.KCLCompileException_Ew: "Compile",
    ErrEwcode.KCLRuntimeException_Ew: "Runtime",
    ErrEwcode.KCLAttributeException_Ew: "An attribute exception occurs",
    ErrEwcode.KCLSchemaException_Ew: "A schema exception occurs",
    ErrEwcode.KCLMixinException_Ew: "A mixin exception occurs",
    ErrEwcode.KCLInheritException_Ew: "An inherit exception occurs",
    ErrEwcode.KCLImportException_Ew: "An import exception occurs",
    ErrEwcode.KCLTypeException_Ew: "A type exception occurs",
    ErrEwcode.KCLDecoratorException_Ew: "A decorator exception occurs",
    ErrEwcode.KCLArgumentException_Ew: "An argument exception occurs",
    ErrEwcode.KCLOverflowException_Ew: "An overflow exception occurs",
    ErrEwcode.KCLComplingException_Ew: "An compling exception occurs",
    ErrEwcode.KCLRunningException_Ew: "An running exception occurs",
    ErrEwcode.KCLDeprecatedException_Ew: "A deprecated exception occurs",
    ErrEwcode.KCLDocException_Ew: "A doc exception occurs",
    ErrEwcode.KCLImmutableException_Ew: "A Immutable exception occurs",
    ErrEwcode.InvalidSyntax_Ew: "Invalid syntax",
    ErrEwcode.KCLTabError_Ew: "Tab Error",
    ErrEwcode.KCLIndentationError_Ew: "Indentation Error",
    ErrEwcode.CannotFindModule_Ew: "Cannot find the module",
    ErrEwcode.FailedLoadModule_Ew: "Failed to load module",
    ErrEwcode.RecursiveLoad_Ew: "Recursively loading module",
    ErrEwcode.FloatOverflow_Ew: "Float overflow",
    ErrEwcode.FloatUnderflow_Ew: "Float underflow",
    ErrEwcode.IntOverflow_Ew: "Integer overflow",
    ErrEwcode.InvalidDocstring_Ew: "Invalid docstring",
    ErrEwcode.Deprecated_Ew: "Deprecated error",
    ErrEwcode.DeprecatedWarning_Ew: "Deprecated warning",
    ErrEwcode.UnKnownDecorator_Ew: "UnKnown decorator",
    ErrEwcode.InvalidDecoratorTarget_Ew: "Invalid Decorator Target",
    ErrEwcode.MixinNamingError_Ew: "Illegal mixin naming",
    ErrEwcode.MixinStructureIllegal_Ew: "Illegal mixin structure",
    ErrEwcode.SchemaCheckFailure_Ew: "Schema check is failed to check condition",
    ErrEwcode.CannotAddMembersComplieError_Ew: "Cannot add members to a schema",
    ErrEwcode.CannotAddMembersRuntimeError_Ew: "Cannot add members to a schema",
    ErrEwcode.IndexSignatureError_Ew: "Invalid index signature",
    ErrEwcode.TypeRuntimeError_Ew: "The type got is inconsistent with the type expected",
    ErrEwcode.TypeComplieError_Ew: "The type got is inconsistent with the type expected",
    ErrEwcode.CompileError_Ew: "A complie error occurs during compiling",
    ErrEwcode.SelectorError_Ew: "Selector Error",
    ErrEwcode.KCLNameError_Ew: "Name Error",
    ErrEwcode.KCLValueError_Ew: "Value Error",
    ErrEwcode.KCLKeyError_Ew: "Key Error",
    ErrEwcode.UniqueKeyError_Ew: "Unique key error",
    ErrEwcode.KCLAttributeComplieError_Ew: "Attribute error occurs during compiling",
    ErrEwcode.KCLAttributeRuntimeError_Ew: "Attribute error occurs at runtime",
    ErrEwcode.IllegalAttributeError_Ew: "Illegal attribute",
    ErrEwcode.MultiInheritError_Ew: "Multiple inheritance is illegal",
    ErrEwcode.CycleInheritError_Ew: "Cycle Inheritance is illegal",
    ErrEwcode.IllegalInheritError_Ew: "Illegal inheritance",
    ErrEwcode.IllegalArgumentRuntimeError_Ew: "Illegal command line argument at runtime",
    ErrEwcode.IllegalArgumentComplieError_Ew: "Illegal argument during compiling",
    ErrEwcode.IllegalArgumentSyntaxError_Ew: "Illegal argument syntax",
    ErrEwcode.EvaluationError_Ew: "Evaluation failure",
    ErrEwcode.InvalidFormatSpec_Ew: "Invalid format specification",
    ErrEwcode.KCLAssertionError_Ew: "Assertion failure",
    ErrEwcode.ImmutableCompileError_Ew: "Immutable variable is modified during compiling",
    ErrEwcode.ImmutableRuntimeError_Ew: "Immutable variable is modified at runtime",
    ErrEwcode.KCLRecursionError_Ew: "Recursively reference",
    ErrEwcode.PlanError_Ew: "Plan Error",
}

ErrArgMsgDefault_EN: dict = {
    ErrEwcode.KCLException_Ew: "Exception",
    ErrEwcode.KCLError_Ew: "Error",
    ErrEwcode.KCLWarning_Ew: "Warning",
    ErrEwcode.KCLSyntaxException_Ew: "Syntax",
    ErrEwcode.KCLCompileException_Ew: "Complie",
    ErrEwcode.KCLRuntimeException_Ew: "Runtime",
    ErrEwcode.KCLAttributeException_Ew: "An attribute exception occurs",
    ErrEwcode.KCLSchemaException_Ew: "A schema exception occurs",
    ErrEwcode.KCLMixinException_Ew: "A mixin exception occurs",
    ErrEwcode.KCLInheritException_Ew: "An inherit exception occurs",
    ErrEwcode.KCLImportException_Ew: "An import exception occurs",
    ErrEwcode.KCLTypeException_Ew: "A type exception occurs",
    ErrEwcode.KCLDecoratorException_Ew: "A decorator exception occurs",
    ErrEwcode.KCLArgumentException_Ew: "An argument exception occurs",
    ErrEwcode.KCLOverflowException_Ew: "An overflow exception occurs",
    ErrEwcode.KCLComplingException_Ew: "An compling exception occurs",
    ErrEwcode.KCLRunningException_Ew: "An running exception occurs",
    ErrEwcode.KCLDeprecatedException_Ew: "A deprecated exception occurs",
    ErrEwcode.KCLDocException_Ew: "A doc exception occurs",
    ErrEwcode.KCLImmutableException_Ew: "A Immutable exception occurs",
    ErrEwcode.InvalidSyntax_Ew: "Invalid syntax",
    ErrEwcode.KCLTabError_Ew: "Inconsistent use of tabs and spaces in indentation",
    ErrEwcode.KCLIndentationError_Ew: "Indentation Error",
    ErrEwcode.CannotFindModule_Ew: "Cannot find the module",
    ErrEwcode.FailedLoadModule_Ew: "Failed to load module",
    ErrEwcode.RecursiveLoad_Ew: "Recursively loading module",
    ErrEwcode.FloatOverflow_Ew: "Float overflow",
    ErrEwcode.FloatUnderflow_Ew: "Float underflow",
    ErrEwcode.IntOverflow_Ew: "Integer overflow",
    ErrEwcode.InvalidDocstring_Ew: "Invalid docstring",
    ErrEwcode.Deprecated_Ew: "Deprecated error",
    ErrEwcode.DeprecatedWarning_Ew: "Deprecated warning",
    ErrEwcode.UnKnownDecorator_Ew: "UnKnown decorator",
    ErrEwcode.InvalidDecoratorTarget_Ew: "Invalid Decorator Target",
    ErrEwcode.MixinNamingError_Ew: "Illegal mixin naming",
    ErrEwcode.MixinStructureIllegal_Ew: "Illegal mixin structure",
    ErrEwcode.SchemaCheckFailure_Ew: "Check failed on check conditions",
    ErrEwcode.CannotAddMembersComplieError_Ew: "Cannot add members to a schema",
    ErrEwcode.CannotAddMembersRuntimeError_Ew: "Cannot add members to a schema",
    ErrEwcode.IndexSignatureError_Ew: "Invalid index signature",
    ErrEwcode.TypeRuntimeError_Ew: "The type got is inconsistent with the type expected",
    ErrEwcode.TypeComplieError_Ew: "The type got is inconsistent with the type expected",
    ErrEwcode.CompileError_Ew: "A complie error occurs during compiling",
    ErrEwcode.SelectorError_Ew: "Selector Error",
    ErrEwcode.KCLNameError_Ew: "Name Error",
    ErrEwcode.KCLValueError_Ew: "Value Error",
    ErrEwcode.KCLKeyError_Ew: "Key Error",
    ErrEwcode.UniqueKeyError_Ew: "Unique key error",
    ErrEwcode.KCLAttributeComplieError_Ew: "Attribute error occurs during compiling",
    ErrEwcode.KCLAttributeRuntimeError_Ew: "Attribute error occurs at runtime",
    ErrEwcode.IllegalAttributeError_Ew: "Illegal attribute",
    ErrEwcode.MultiInheritError_Ew: "Multiple inheritance is illegal",
    ErrEwcode.CycleInheritError_Ew: "Cycle Inheritance is illegal",
    ErrEwcode.IllegalInheritError_Ew: "Illegal inheritance",
    ErrEwcode.IllegalArgumentRuntimeError_Ew: "Illegal command line argument at runtime",
    ErrEwcode.IllegalArgumentComplieError_Ew: "Illegal argument during compiling",
    ErrEwcode.IllegalArgumentSyntaxError_Ew: "Illegal argument syntax",
    ErrEwcode.EvaluationError_Ew: "Evaluation failure",
    ErrEwcode.InvalidFormatSpec_Ew: "Invalid format specification",
    ErrEwcode.KCLAssertionError_Ew: "Assertion failure",
    ErrEwcode.ImmutableCompileError_Ew: "Immutable variable is modified during compiling",
    ErrEwcode.ImmutableRuntimeError_Ew: "Immutable variable is modified at runtime",
    ErrEwcode.KCLRecursionError_Ew: "maximum recursion depth exceeded",
    ErrEwcode.PlanError_Ew: "Plan Error",
}

ErrName_EN: dict = {
    ErrEwcode.KCLException_Ew: "KCLException",
    ErrEwcode.KCLError_Ew: "KCLError",
    ErrEwcode.KCLWarning_Ew: "KCLWarning",
    ErrEwcode.KCLSyntaxException_Ew: "KCLSyntaxException",
    ErrEwcode.KCLCompileException_Ew: "KCLCompileException",
    ErrEwcode.KCLRuntimeException_Ew: "KCLRuntimeException",
    ErrEwcode.KCLAttributeException_Ew: "KCLAttributeException",
    ErrEwcode.KCLSchemaException_Ew: "KCLSchemaException",
    ErrEwcode.KCLMixinException_Ew: "KCLMixinException",
    ErrEwcode.KCLInheritException_Ew: "KCLInheritException",
    ErrEwcode.KCLImportException_Ew: "KCLImportException",
    ErrEwcode.KCLTypeException_Ew: "KCLTypeException",
    ErrEwcode.KCLDecoratorException_Ew: "KCLDecoratorException",
    ErrEwcode.KCLArgumentException_Ew: "KCLArgumentException",
    ErrEwcode.KCLOverflowException_Ew: "KCLOverflowException",
    ErrEwcode.KCLComplingException_Ew: "KCLComplingException",
    ErrEwcode.KCLRunningException_Ew: "KCLRunningException",
    ErrEwcode.KCLDeprecatedException_Ew: "KCLDeprecatedException",
    ErrEwcode.KCLDocException_Ew: "KCLDocException",
    ErrEwcode.KCLImmutableException_Ew: "KCLImmutableException",
    ErrEwcode.InvalidSyntax_Ew: "InvalidSyntax",
    ErrEwcode.KCLTabError_Ew: "KCLTabError",
    ErrEwcode.KCLIndentationError_Ew: "KCLIndentationError",
    ErrEwcode.CannotFindModule_Ew: "CannotFindModule",
    ErrEwcode.FailedLoadModule_Ew: "FailedLoadModule",
    ErrEwcode.RecursiveLoad_Ew: "RecursiveLoad",
    ErrEwcode.FloatOverflow_Ew: "FloatOverflow",
    ErrEwcode.FloatUnderflow_Ew: "FloatUnderflow",
    ErrEwcode.IntOverflow_Ew: "IntOverflow",
    ErrEwcode.InvalidDocstring_Ew: "InvalidDocstring",
    ErrEwcode.Deprecated_Ew: "Deprecated",
    ErrEwcode.DeprecatedWarning_Ew: "DeprecatedWarning",
    ErrEwcode.UnKnownDecorator_Ew: "UnKnownDecorator",
    ErrEwcode.InvalidDecoratorTarget_Ew: "InvalidDecoratorTarget",
    ErrEwcode.MixinNamingError_Ew: "MixinNamingError",
    ErrEwcode.MixinStructureIllegal_Ew: "MixinStructureIllegal",
    ErrEwcode.SchemaCheckFailure_Ew: "SchemaCheckFailure",
    ErrEwcode.CannotAddMembersComplieError_Ew: "CannotAddMembersComplieError",
    ErrEwcode.CannotAddMembersRuntimeError_Ew: "CannotAddMembersRuntimeError",
    ErrEwcode.IndexSignatureError_Ew: "IndexSignatureError",
    ErrEwcode.TypeRuntimeError_Ew: "TypeRuntimeError",
    ErrEwcode.TypeComplieError_Ew: "TypeComplieError",
    ErrEwcode.CompileError_Ew: "CompileError",
    ErrEwcode.SelectorError_Ew: "SelectorError",
    ErrEwcode.KCLNameError_Ew: "KCLNameError",
    ErrEwcode.KCLValueError_Ew: "KCLValueError",
    ErrEwcode.KCLKeyError_Ew: "KCLKeyError",
    ErrEwcode.UniqueKeyError_Ew: "UniqueKeyError",
    ErrEwcode.KCLAttributeComplieError_Ew: "KCLAttributeComplieError",
    ErrEwcode.KCLAttributeRuntimeError_Ew: "KCLAttributeRuntimeError",
    ErrEwcode.IllegalAttributeError_Ew: "IllegalAttributeError",
    ErrEwcode.MultiInheritError_Ew: "MultiInheritError",
    ErrEwcode.CycleInheritError_Ew: "CycleInheritError",
    ErrEwcode.IllegalInheritError_Ew: "IllegalInheritError",
    ErrEwcode.IllegalArgumentRuntimeError_Ew: "IllegalArgumentRuntimeError",
    ErrEwcode.IllegalArgumentComplieError_Ew: "IllegalArgumentComplieError",
    ErrEwcode.IllegalArgumentSyntaxError_Ew: "IllegalArgumentSyntaxError",
    ErrEwcode.EvaluationError_Ew: "EvaluationError",
    ErrEwcode.InvalidFormatSpec_Ew: "InvalidFormatSpec",
    ErrEwcode.KCLAssertionError_Ew: "KCLAssertionError",
    ErrEwcode.ImmutableCompileError_Ew: "ImmutableCompileError",
    ErrEwcode.ImmutableRuntimeError_Ew: "ImmutableRuntimeError",
    ErrEwcode.KCLRecursionError_Ew: "KCLRecursionError",
    ErrEwcode.PlanError_Ew: "PlanError",
}


class ErrMsgLang:
    EN_US_UTF_8 = "c.utf8"


class ErrMsg(metaclass=ABCMeta):
    @abstractmethod
    def get_defaule_ewcode(self) -> str:
        pass

    @abstractmethod
    def get_err_cate_ewcode(self, err_id: str) -> str:
        pass

    @abstractmethod
    def get_err_type_ewcode(self, err_id: str) -> str:
        pass

    @abstractmethod
    def get_err_cate_msg_by_errid(self, err_id: str) -> str:
        pass

    @abstractmethod
    def get_err_type_msg_by_errid(self, err_id: str) -> str:
        pass

    @abstractmethod
    def get_err_msg_by_errid(self, err_id: str) -> str:
        pass

    @abstractmethod
    def get_defaule_arg_msg_by_errid(self, err_id: str) -> str:
        pass

    @abstractmethod
    def get_err_code_by_errid(self, err_id: str) -> str:
        pass

    @abstractmethod
    def is_warning(self, err_id: str) -> bool:
        pass

    @abstractmethod
    def is_syntax_error(self, err_id: str) -> bool:
        pass

    @abstractmethod
    def is_compiler_error(self, err_id: str) -> bool:
        pass

    @abstractmethod
    def is_runtime_error(self, err_id: str) -> bool:
        pass

    @abstractmethod
    def get_err_name_by_ewcode(self, err_id: str):
        pass


class KCLErrMsg_EN(ErrMsg):
    @PostCheck(lambda result: ErrEwcode.contains(result))
    def get_defaule_ewcode(self) -> str:
        return ErrEwcode.KCLException_Ew

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: ErrEwcode.contains(result))
    @PostSimpleExprCheck(
        (lambda inputs, result: result == inputs["err_id"][:1] + "0000"), ["err_id"]
    )
    def get_err_cate_ewcode(self, err_id: str) -> str:
        return err_id[:1] + "0000"

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: ErrEwcode.contains(result))
    @PostSimpleExprCheck(
        (lambda inputs, result: result == "0" + inputs["err_id"][1:2] + "000"),
        ["err_id"],
    )
    def get_err_type_ewcode(self, err_id: str) -> str:
        return "0" + err_id[1:2] + "000"

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: result in ErrMsgContent_EN.values())
    def get_err_cate_msg_by_errid(self, err_id: str) -> str:
        return ErrMsgContent_EN[self.get_err_cate_ewcode(err_id)]

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: result in ErrMsgContent_EN.values())
    def get_err_type_msg_by_errid(self, err_id: str) -> str:
        return ErrMsgContent_EN[self.get_err_type_ewcode(err_id)]

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: result in ErrMsgContent_EN.values())
    def get_err_msg_by_errid(self, err_id: str) -> str:
        return ErrMsgContent_EN[err_id]

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: result in ErrArgMsgDefault_EN.values())
    def get_defaule_arg_msg_by_errid(self, err_id: str) -> str:
        return ErrArgMsgDefault_EN[err_id]

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: CheckRules.check_type_not_none(result, str))
    @PostSimpleExprCheck(
        (lambda inputs, result: result == "[" + inputs["err_id"] + "]"), ["err_id"]
    )
    def get_err_code_by_errid(self, err_id: str) -> str:
        return "[" + err_id + "]"

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: CheckRules.check_type_not_none(result, bool))
    def is_warning(self, err_id: str) -> bool:
        return err_id[:1] == WARNING

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: CheckRules.check_type_not_none(result, bool))
    def is_syntax_error(self, err_id: str) -> bool:
        return err_id[1:2] == SYNTAX

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: CheckRules.check_type_not_none(result, bool))
    def is_compiler_error(self, err_id: str) -> bool:
        return err_id[1:2] == COMPLIER

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: CheckRules.check_type_not_none(result, bool))
    def is_runtime_error(self, err_id: str) -> bool:
        return err_id[1:2] == RUNTIME

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: result in ErrName_EN.values())
    def get_err_name_by_ewcode(self, err_id: str) -> str:
        return ErrName_EN[err_id]


class KCLErrMsgManager(ErrMsg):
    _instance = None

    def __new__(cls):
        if cls._instance is None:
            cls._instance = object.__new__(cls)
        return cls._instance

    def __init__(self, lang: ErrMsgLang = ErrMsgLang.EN_US_UTF_8):
        self.ALL_KCL_ERROR_MSGS: dict = {ErrMsgLang.EN_US_UTF_8: KCLErrMsg_EN()}
        self.lang = lang
        self.KCL_ERROR_MSG = self.ALL_KCL_ERROR_MSGS[self.lang]

    @PreCheck((lambda v: CheckRules.check_locale(v)), "lang")
    def switch_lang(self, lang: str):
        try:
            self.KCL_ERROR_MSG = self.ALL_KCL_ERROR_MSGS[lang]
        except KeyError:
            print(
                f"KCLVM does not support '{lang}', "
                f"KCLVM have automatically switched to {ErrMsgLang.EN_US_UTF_8}",
                file=sys.stderr,
            )
            self.lang = ErrMsgLang.EN_US_UTF_8
            self.KCL_ERROR_MSG = self.ALL_KCL_ERROR_MSGS[self.lang]

    @PreCheck((lambda v: CheckRules.check_type_not_none(v, ErrMsg)), "err_msg")
    @PreCheck((lambda v: CheckRules.check_locale(v)), "lang")
    def append_lang(self, lang: str, err_msg: ErrMsg):
        if lang in self.ALL_KCL_ERROR_MSGS.keys():
            print(
                f"KCLVM currently supports {lang}, "
                f"If you want to change the language pack, "
                f"please use the method update_lang",
                file=sys.stderr,
            )
            return
        else:
            self.ALL_KCL_ERROR_MSGS[lang] = err_msg

    @PreCheck((lambda v: CheckRules.check_type_not_none(v, ErrMsg)), "err_msg")
    @PreCheck((lambda v: CheckRules.check_locale(v)), "lang")
    def update_lang(self, lang: str, err_msg: ErrMsg):
        self.ALL_KCL_ERROR_MSGS[lang] = err_msg

    @PostCheck(lambda result: CheckRules.check_type_not_none(result, str))
    @PostCheck(lambda result: result == ErrEwcode.KCLException_Ew)
    def get_defaule_ewcode(self) -> str:
        return self.KCL_ERROR_MSG.get_defaule_ewcode()

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: CheckRules.check_type_not_none(result, str))
    @PostSimpleExprCheck(
        (lambda inputs, result: result == inputs["err_id"][:1] + "0000"), ["err_id"]
    )
    def get_err_cate_ewcode(self, err_id: str) -> str:
        return self.KCL_ERROR_MSG.get_err_cate_ewcode(err_id)

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: CheckRules.check_type_not_none(result, str))
    @PostSimpleExprCheck(
        (lambda inputs, result: result == "0" + inputs["err_id"][1:2] + "000"),
        ["err_id"],
    )
    def get_err_type_ewcode(self, err_id: str) -> str:
        return self.KCL_ERROR_MSG.get_err_type_ewcode(err_id)

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: CheckRules.check_type_not_none(result, str))
    def get_err_cate_msg_by_errid(self, err_id: str) -> str:
        return self.KCL_ERROR_MSG.get_err_cate_msg_by_errid(err_id)

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: CheckRules.check_type_not_none(result, str))
    def get_err_type_msg_by_errid(self, err_id: str) -> str:
        return self.KCL_ERROR_MSG.get_err_type_msg_by_errid(err_id)

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: CheckRules.check_type_not_none(result, str))
    def get_err_msg_by_errid(self, err_id: str) -> str:
        return self.KCL_ERROR_MSG.get_err_msg_by_errid(err_id)

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: CheckRules.check_type_not_none(result, str))
    def get_defaule_arg_msg_by_errid(self, err_id: str) -> str:
        return self.KCL_ERROR_MSG.get_defaule_arg_msg_by_errid(err_id)

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: CheckRules.check_type_not_none(result, str))
    @PostSimpleExprCheck(
        (lambda inputs, result: result == "[" + inputs["err_id"] + "]"), ["err_id"]
    )
    def get_err_code_by_errid(self, err_id: str) -> str:
        return self.KCL_ERROR_MSG.get_err_code_by_errid(err_id)

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: CheckRules.check_type_not_none(result, bool))
    def is_warning(self, err_id: str) -> bool:
        return self.KCL_ERROR_MSG.is_warning(err_id)

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: CheckRules.check_type_not_none(result, bool))
    def is_syntax_error(self, err_id: str) -> bool:
        return self.KCL_ERROR_MSG.is_syntax_error(err_id)

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: CheckRules.check_type_not_none(result, bool))
    def is_compiler_error(self, err_id: str) -> bool:
        return self.KCL_ERROR_MSG.is_compiler_error(err_id)

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: CheckRules.check_type_not_none(result, bool))
    def is_runtime_error(self, err_id: str) -> bool:
        return self.KCL_ERROR_MSG.is_runtime_error(err_id)

    @PreCheck((lambda v: ErrEwcode.contains(v)), "err_id")
    @PostCheck(lambda result: CheckRules.check_type_not_none(result, str))
    def get_err_name_by_ewcode(self, err_id: str):
        return self.KCL_ERROR_MSG.get_err_name_by_ewcode(err_id)


KCL_ERR_MSG = KCLErrMsgManager()
