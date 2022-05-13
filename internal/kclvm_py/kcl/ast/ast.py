"""The `ast` file contains the definitions of all KCL AST nodes
and operators and all AST nodes are derived from the `AST` class.
The main structure of a KCL program is as follows:

┌─────────────────────────────────────────────────────────────────┐
│                             Program                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │   Main Package  │  │     Package1    │  │     Package2    │  │
│  │  ┌───────────┐  │  │  ┌───────────┐  │  │  ┌───────────┐  │  │
│  │  │  Module1  │  │  │  │  Module1  │  │  │  │  Module1  │  │  │
│  │  └───────────┘  │  │  └───────────┘  │  │  └───────────┘  │  │
│  │  ┌───────────┐  │  │  ┌───────────┐  │  │  ┌───────────┐  │  │
│  │  │  Module2  │  │  │  │  Module2  │  │  │  │  Module2  │  │  │
│  │  └───────────┘  │  │  └───────────┘  │  │  └───────────┘  │  │
│  │  ┌───────────┐  │  │  ┌───────────┐  │  │  ┌───────────┐  │  │
│  │  │    ...    │  │  │  │    ...    │  │  │  │    ...    │  │  │
│  │  └───────────┘  │  │  └───────────┘  │  │  └───────────┘  │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────────┘

A single KCL file represents a module, which records file information,
package path information, and module document information, which is
mainly composed of all the statements in the KCL file.

The combination of multiple KCL files is regarded as a complete KCL
Program. For example, a single KCL file can be imported into KCL
files in other packages through statements such as import. Therefore,
the Program is composed of multiple modules, and each module is
associated with it. Corresponding to the package path.

:note: When the definition of any AST node is modified or the AST node
is added/deleted, it is necessary to modify the corresponding processing
in the compiler and regenerate the walker code.
:copyright: Copyright 2020 The KCL Authors. All rights reserved.
"""

import os
import json
import hashlib
import typing

from enum import Enum
from abc import ABC
from typing import List, Optional, Union, Dict
from pathlib import PosixPath

import kclvm.kcl.ast.lark_token as lark_token
import kclvm.kcl.ast as ast
import kclvm.internal.util.check_utils as check_utils

from .lark_token import TokenValue


class CmdArgSpec:
    """KCL command line argument spec, e.g. `kcl main.k -D name=value`"""

    def __init__(self, *, name: str = "", value: any = None):
        self.name: str = name
        self.value: any = value


class OverrideAction(Enum):
    CREATE_OR_UPDATE = "CreateOrUpdate"
    DELETE = "Delete"


class CmdOverrideSpec:
    """KCL command line override spec, e.g. `kcl main.k -O pkgpath:path.to.field=field_value`"""

    def __init__(
        self,
        *,
        pkgpath: str = "",
        field_path: str = "",
        field_value: str = "",
        action: OverrideAction = OverrideAction.CREATE_OR_UPDATE,
    ):
        self.pkgpath: str = pkgpath
        self.field_path: str = field_path
        self.field_value: str = field_value
        self.action: OverrideAction = action


class LarkToken(lark_token.LarkToken):
    @staticmethod
    def is_string(token: str):
        return token in [
            LarkToken.L_STRING,
            LarkToken.L_LONG_STRING,
        ]

    @staticmethod
    def is_int_number(token: str):
        return token in [
            LarkToken.L_DEC_NUMBER,
            LarkToken.L_HEX_NUMBER,
            LarkToken.L_BIN_NUMBER,
            LarkToken.L_OCT_NUMBER,
        ]

    @staticmethod
    def is_float_number(token: str):
        return token == LarkToken.L_FLOAT_NUMBER

    @staticmethod
    def is_name_constant(token: str):
        return token in [
            LarkToken.L_TRUE,
            LarkToken.L_FALSE,
            LarkToken.L_NONE,
            LarkToken.L_UNDEFINED,
        ]

    @staticmethod
    def get_token_value(token: str) -> str:
        return LarkToken.LL_token_str_value_map[token]


class BinOp(Enum):
    """BinOp is the set of all binary operators in KCL."""

    Add = LarkToken.L_PLUS
    Sub = LarkToken.L_MINUS
    Mul = LarkToken.L_MULTIPLY
    Div = LarkToken.L_DIVIDE
    Mod = LarkToken.L_MOD
    Pow = LarkToken.L_DOUBLE_STAR
    LShift = LarkToken.L_SHIFT_LEFT
    RShift = LarkToken.L_SHIFT_RIGHT
    BitOr = LarkToken.L_OR
    BitXor = LarkToken.L_XOR
    BitAnd = LarkToken.L_AND
    FloorDiv = LarkToken.L_DOUBLE_DIVIDE
    As = LarkToken.L_AS

    And = LarkToken.L_L_AND  # True and False
    Or = LarkToken.L_L_OR  # True or False

    @classmethod
    def enum_value_list(cls) -> [str]:
        return list(map(lambda c: c.value, cls))

    @classmethod
    def enum_key_list(cls) -> [str]:
        return list(map(lambda c: c, cls))


class AugOp(Enum):
    Assign = LarkToken.L_ASSIGN
    Add = LarkToken.L_COMP_PLUS
    Sub = LarkToken.L_COMP_MINUS
    Mul = LarkToken.L_COMP_MULTIPLY
    Div = LarkToken.L_COMP_DIVIDE
    Mod = LarkToken.L_COMP_MOD
    Pow = LarkToken.L_COMP_DOUBLE_STAR
    LShift = LarkToken.L_COMP_SHIFT_LEFT
    RShift = LarkToken.L_COMP_SHIFT_RIGHT
    BitOr = LarkToken.L_COMP_OR
    BitXor = LarkToken.L_COMP_XOR
    BitAnd = LarkToken.L_COMP_AND
    FloorDiv = LarkToken.L_COMP_DOUBLE_DIVIDE

    @classmethod
    def enum_value_list(cls) -> [str]:
        return list(map(lambda c: c.value, cls))

    @classmethod
    def enum_key_list(cls) -> [str]:
        return list(map(lambda c: c, cls))


class UnaryOp(Enum):
    UAdd = LarkToken.L_PLUS
    USub = LarkToken.L_MINUS
    Invert = LarkToken.L_NOT
    Not = LarkToken.L_L_NOT

    @classmethod
    def enum_value_list(cls) -> [str]:
        return list(map(lambda c: c.value, cls))

    @classmethod
    def enum_key_list(cls) -> [str]:
        return list(map(lambda c: c, cls))


def judge_compare_op(optype: str) -> bool:
    return optype in CmpOp.enum_value_list()


CMP_OP_VALUE_LIST = [
    LarkToken.L_EQUAL_TO,
    LarkToken.L_NOT_EQUAL_TO,
    LarkToken.L_LESS_THAN,
    LarkToken.L_LESS_THAN_OR_EQUAL_TO,
    LarkToken.L_GREATER_THAN,
    LarkToken.L_GREATER_THAN_OR_EQUAL_TO,
    LarkToken.L_IS,
    LarkToken.L_IN,
    LarkToken.L_L_NOT,
]


class CmpOp(Enum):
    Eq = LarkToken.L_EQUAL_TO
    NotEq = LarkToken.L_NOT_EQUAL_TO
    Lt = LarkToken.L_LESS_THAN
    LtE = LarkToken.L_LESS_THAN_OR_EQUAL_TO
    Gt = LarkToken.L_GREATER_THAN
    GtE = LarkToken.L_GREATER_THAN_OR_EQUAL_TO
    Is = LarkToken.L_IS
    In = LarkToken.L_IN

    Not = LarkToken.L_L_NOT

    IsNot = f"{LarkToken.L_IS} {LarkToken.L_L_NOT}"  # "IS NOT"
    NotIn = f"{LarkToken.L_L_NOT} {LarkToken.L_IN}"  # "NOT IN"

    @classmethod
    def enum_value_list(cls) -> [str]:
        return CMP_OP_VALUE_LIST

    @classmethod
    def enum_key_list(cls) -> [str]:
        return list(map(lambda c: c, cls))


class ExprContext(Enum):
    LOAD = "LOAD"
    STORE = "STORE"
    DEL = "DEL"
    AUGLOAD = "AUGLOAD"
    AUGSTORE = "AUGSTORE"
    PARAM = "PARAM"

    @classmethod
    def enum_value_list(cls) -> [str]:
        return list(map(lambda c: c.value, cls))


AST_ENUM_LIST = {
    "BinOp": BinOp,
    "AugOp": AugOp,
    "UnaryOp": UnaryOp,
    "CmpOp": CmpOp,
    "ExprContext": ExprContext,
    "OverrideAction": OverrideAction,
}

OPERATOR_VALUE_MAP = {
    AugOp.Assign: TokenValue.ASSIGN,
    AugOp.Add: TokenValue.COMP_PLUS,
    AugOp.Sub: TokenValue.COMP_MINUS,
    AugOp.Mul: TokenValue.COMP_MULTIPLY,
    AugOp.Div: TokenValue.COMP_DIVIDE,
    AugOp.Mod: TokenValue.COMP_MOD,
    AugOp.Pow: TokenValue.COMP_DOUBLE_STAR,
    AugOp.LShift: TokenValue.COMP_SHIFT_LEFT,
    AugOp.RShift: TokenValue.COMP_SHIFT_RIGHT,
    AugOp.BitOr: TokenValue.COMP_OR,
    AugOp.BitXor: TokenValue.COMP_XOR,
    AugOp.BitAnd: TokenValue.COMP_AND,
    AugOp.FloorDiv: TokenValue.COMP_DOUBLE_DIVIDE,
    BinOp.Add: TokenValue.PLUS,
    BinOp.Sub: TokenValue.MINUS,
    BinOp.Mul: TokenValue.MULTIPLY,
    BinOp.Div: TokenValue.DIVIDE,
    BinOp.Mod: TokenValue.MOD,
    BinOp.Pow: TokenValue.DOUBLE_STAR,
    BinOp.LShift: TokenValue.SHIFT_LEFT,
    BinOp.RShift: TokenValue.SHIFT_RIGHT,
    BinOp.BitOr: TokenValue.OR,
    BinOp.BitXor: TokenValue.XOR,
    BinOp.BitAnd: TokenValue.AND,
    BinOp.FloorDiv: TokenValue.DOUBLE_DIVIDE,
    BinOp.And: TokenValue.L_AND,
    BinOp.Or: TokenValue.L_OR,
    BinOp.As: TokenValue.AS,
    CmpOp.Eq: TokenValue.EQUAL_TO,
    CmpOp.NotEq: TokenValue.NOT_EQUAL_TO,
    CmpOp.Lt: TokenValue.LESS_THAN,
    CmpOp.LtE: TokenValue.LESS_THAN_OR_EQUAL_TO,
    CmpOp.Gt: TokenValue.GREATER_THAN,
    CmpOp.GtE: TokenValue.GREATER_THAN_OR_EQUAL_TO,
    CmpOp.Is: TokenValue.IS,
    CmpOp.In: TokenValue.IN,
    CmpOp.Not: TokenValue.NOT,
    CmpOp.IsNot: " ".join([TokenValue.IS, TokenValue.L_NOT]),
    CmpOp.NotIn: " ".join([TokenValue.L_NOT, TokenValue.IN]),
    UnaryOp.UAdd: TokenValue.PLUS,
    UnaryOp.USub: TokenValue.MINUS,
    UnaryOp.Invert: TokenValue.NOT,
    UnaryOp.Not: TokenValue.L_NOT,
}


class Position:
    """Position describes an arbitrary source position including the filename,
    line, and column location.

    A Position is valid if the line number is > 0.
    The line and column are both 1 based.
    """

    def __init__(self, filename: str = None, line: int = None, column: int = None):
        self.filename: str = filename
        self.line: int = line
        self.column: int = column

    def is_valid(self) -> bool:
        return self.filename is not None and self.line is not None and self.line > 0

    def less(self, pos: "Position") -> bool:
        if not self.is_valid() or not pos or not pos.is_valid():
            return False
        if self.filename != pos.filename:
            return False
        if self.line < pos.line:
            return True
        if self.line == pos.line:
            return self.column < pos.column
        return False

    def less_equal(self, pos: "Position") -> bool:
        if not self.is_valid() or not pos or not pos.is_valid():
            return False
        if self.less(pos):
            return True
        return self == pos

    def __eq__(self, other: "Position") -> bool:
        return (
            self.filename == other.filename
            and self.line == other.line
            and self.column == other.column
        )

    def __str__(self) -> str:
        return f"<{self.filename}, ({self.line}, {self.column})>"


class AST:
    """
    All KCL node types implement the KCL AST interface
    """

    _line_offset: int = 0
    _column_offset: int = 0

    def __init__(
        self,
        line: Optional[int] = 0,
        column: Optional[int] = 0,
        end_line: Optional[int] = 0,
        end_column: Optional[int] = 0,
    ) -> None:
        self.filename: str = None
        self.relative_filename: str = None
        self.line: int = line
        self.column: int = column
        self.end_line: int = end_line
        self.end_column: int = end_column
        self.parent: Optional[AST] = None

    def __str__(self) -> str:
        return f"<{self.type}, ({self.line}, {self.column})>"

    def __repr__(self) -> str:
        return self.__str__()

    def get_line(self) -> int:
        """
        Get the node line, which is 1 based
        """
        return self.line

    def get_column(self) -> int:
        """
        Get the node column, which is 1 based
        """
        return self.column

    def get_end_line(self) -> int:
        """
        Get the node end_line
        """
        return self.end_line

    def get_end_column(self) -> int:
        """
        Get the node end_column
        """
        return self.end_column

    def set_ast_position(self, node, filename=None):
        import kclvm.compiler.parser.lark_pb2 as lark_pb

        check_utils.check_type_not_none(node, AST, lark_pb.Tree)
        check_utils.check_type_allow_none(filename, str, PosixPath)
        self.filename = filename if isinstance(node, lark_pb.Tree) else node.filename
        self.line = node.line + self._line_offset
        self.column = node.column + self._column_offset
        self.end_line = node.end_line + self._line_offset
        self.end_column = node.end_column + self._column_offset

        return self

    def set_column(self, column: int):
        assert isinstance(column, int) and column >= 0
        self.column = column

    def set_line(self, line: int):
        assert isinstance(line, int) and line >= 0
        self.line = line

    def set_end_line_column(self, ast_node):
        assert ast_node and isinstance(ast_node, AST)
        self.end_line = ast_node.end_line
        self.end_column = ast_node.end_column

    def set_end_line(self, end_line: int):
        assert isinstance(end_line, int) and end_line >= 0
        self.end_line = end_line

    def set_end_column(self, end_column: int):
        assert isinstance(end_column, int) and end_column >= 0
        self.end_column = end_column

    def offset_column(self, offset_column: int):
        assert isinstance(offset_column, int)
        self.column += offset_column

    def offset_line(self, offset_line: int):
        assert isinstance(offset_line, int)
        self.line += offset_line

    def offset_end_line(self, offset_end_line: int):
        assert isinstance(offset_end_line, int)
        self.end_line += offset_end_line

    def offset_end_column(self, offset_end_column: int):
        assert isinstance(offset_end_column, int)
        self.end_column += offset_end_column

    @classmethod
    def set_offset(cls, line_offset: int = 0, column_offset=0):
        cls._line_offset = line_offset
        cls._column_offset = column_offset

    @classmethod
    def reset_offset(cls):
        cls._line_offset = 0
        cls._column_offset = 0

    @property
    def type(self) -> str:
        return self.__class__.__name__

    @property
    def pos(self) -> Position:
        return Position(filename=self.filename, line=self.line, column=self.column)

    @property
    def end_pos(self) -> Position:
        return Position(
            filename=self.filename, line=self.end_line, column=self.end_column
        )

    def to_json(self, indent=4, sort_keys=False):
        return json.dumps(
            self,
            default=lambda o: o.value
            if type(o) in AST_ENUM_LIST.values()
            else {k: v for k, v in o.__dict__.items() if k != "parent"},
            indent=indent,
            sort_keys=sort_keys,
        )

    def contains_pos(self, pos: Position) -> bool:
        """
        check if current node contains a position
        :param pos: the given position
        :return: if current node contains the given position
        """
        start_pos = Position(
            filename=self.filename,
            line=self.line,
            column=self.column,
        )
        end_pos = Position(
            filename=self.filename,
            line=self.end_line,
            column=self.end_column,
        )
        return start_pos.less_equal(pos) and pos.less_equal(end_pos)

    def get_children(self) -> List["AST"]:
        def walk_field(inner: typing.Union[typing.List, typing.Dict, AST]):
            if isinstance(inner, list):
                [walk_field(item) for item in inner]
                return
            if isinstance(inner, dict):
                [walk_field(v) for _, v in inner]
                return
            if isinstance(inner, AST):
                children.append(inner)

        children = []
        [walk_field(field) for _, field in ast.iter_fields(self)]
        return children

    def find_leaf_by_pos(self, pos: Position) -> Optional["AST"]:
        if pos and pos.is_valid() and self.contains_pos(pos):
            children = self.get_children()
            if len(children) == 0:
                return self
            else:
                for child in children:
                    leaf = child.find_leaf_by_pos(pos)
                    if leaf:
                        return leaf
        return None

    def find_nearest_parent_by_type(self, tpe: typing.Type["AST"]) -> Optional["AST"]:
        parent = self.parent
        while parent:
            if parent.type == tpe.__name__:
                return typing.cast(tpe, parent)
            parent = parent.parent
        return None


class Stmt(AST):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "Stmt"


class Expr(AST):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "Expr"


class Name(Expr, ABC):
    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
        value: Optional[str] = None,
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "Name"
        self.value: str = value


class TypeAliasStmt(Stmt):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "TypeAliasStmt"
        self.type_name: Optional[Identifier] = None
        self.type_value: Optional[Type] = None


class ExprStmt(Stmt):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "ExprStmt"
        self.exprs: List[Expr] = []


class UnificationStmt(Stmt):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "UnificationStmt"
        self.target: Identifier = None
        self.value: SchemaExpr = None


class AssignStmt(Stmt):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "AssignStmt"
        self.targets: List[Identifier] = []
        self.value: Optional[Expr] = None
        self.type_annotation: str = ""
        self.type_annotation_node: Optional[Type] = None

    def __str__(self):
        return super().__str__()[:-1] + f" targets: {self.targets} value: {self.value}>"


class AugAssignStmt(Stmt):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "AugAssignStmt"
        self.op: Optional[AugOp] = None
        self.target: Optional[Identifier] = None
        self.value: Optional[Expr] = None

    def __str__(self):
        return (
            super().__str__()[:-1]
            + f" target: {self.target} augop: {self.op} value: {self.value}>"
        )


class AssertStmt(Stmt):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "AssertStmt"
        self.test: Optional[Expr] = None
        self.if_cond: Optional[Expr] = None
        self.msg: Optional[Expr] = None


class IfStmt(Stmt):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "IfStmt"
        self.cond: Optional[Expr] = None
        self.body: List[Stmt] = []
        self.elif_cond: List[Expr] = []
        self.elif_body: List[List[Stmt]] = []
        self.else_body: List[Stmt] = []

    def __str__(self):
        return (
            super().__str__()[:-1]
            + f" cond: {self.cond} body: {self.body} elif_cond: {self.elif_cond} elifbody: {self.elif_body} elsebody: {self.else_body}>"
        )


class ImportStmt(Stmt):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "ImportStmt"
        self.path: Optional[str] = None
        self.name: Optional[str] = None
        self.asname: Optional[str] = None
        self.path_nodes: [Name] = []
        self.as_name_node: Optional[Name] = None

        self.rawpath: Optional[str] = None  # only for error message

    def __str__(self):
        return super().__str__()[:-1] + f" name: {self.name}>"

    @property
    def pkg_name(self) -> str:
        return self.asname or self.name


class SchemaIndexSignature(Stmt):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "SchemaIndexSignature"
        self.key_name: Optional[str] = None
        self.key_type: Optional[str] = "str"
        self.value_type: Optional[str] = ""
        self.value: Optional[Expr] = None
        self.any_other: bool = False
        self.name_node: Optional[Name] = None
        self.value_type_node: Optional[Type] = None


class SchemaAttr(Stmt):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "SchemaAttr"
        self.doc: str = ""
        self.name: str = ""
        self.type_str: str = ""
        self.op: Optional[Union[BinOp, AugOp]] = None
        self.value: Optional[Expr] = None
        self.is_optional: bool = False
        self.decorators: List[Decorator] = []
        self.name_node: Optional[Name] = None
        self.type_node: Optional[Type] = None

    def __str__(self):
        return (
            super().__str__()[:-1]
            + f" name: {self.name} type: {self.type_str} value: {self.value}>"
        )


class SchemaStmt(Stmt):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "SchemaStmt"
        self.doc: str = ""
        self.name: str = ""
        self.parent_name: Optional[Identifier] = None
        self.for_host_name: Optional[Identifier] = None
        self.is_mixin: bool = False
        self.is_protocol: bool = False
        self.args: Optional[Arguments] = None
        self.mixins: List[Identifier] = []
        self.body: List[Union[SchemaAttr, Stmt]] = []
        self.decorators: List[Decorator] = []
        self.checks: List[CheckExpr] = []
        self.index_signature: Optional[SchemaIndexSignature] = None
        self.name_node: Optional[Name] = None

    def __str__(self):
        return super().__str__()[:-1] + f" name: {self.name} body: {self.body}>"

    def has_only_attribute_definitions(self) -> bool:
        return not bool(
            self.args
            or self.mixins
            or self.checks
            or self.index_signature
            or any(
                [not isinstance(p, (SchemaAttr, UnificationStmt)) for p in self.body]
            )
        )

    def GetAttrList(self) -> List[SchemaAttr]:
        attr_list = []

        for attr in self.body or []:
            if isinstance(attr, (SchemaAttr, UnificationStmt)):
                attr_list.append(attr)

        return attr_list

    def GetAttrNameList(self) -> List[str]:
        attr_list = self.GetAttrList()
        return [attr.name for attr in attr_list]

    def GetIndexSignatureAttrName(self) -> Optional[str]:
        return self.index_signature.key_name if self.index_signature else None

    def GetStmtList(self) -> List[Stmt]:
        stmt_list = []

        for attr in self.body:
            if not isinstance(attr, SchemaAttr):
                stmt_list.append(attr)

        return stmt_list

    def GetLeftIdentifierList(self):
        """Get schema full attribute list including
        un-exported attributes and relaxed attributes
        """
        attr_list = []

        def loop_body(body: List[Stmt]):
            """Get the l-values recursively and add them into schema attr list"""
            if not body:
                return
            for stmt in body:
                if isinstance(stmt, AssignStmt):
                    for target in stmt.targets:
                        add_name(target.get_first_name())
                elif isinstance(stmt, (AugAssignStmt, UnificationStmt)):
                    add_name(stmt.target.get_first_name())
                elif isinstance(stmt, IfStmt):
                    loop_body(stmt.body)
                    for body in stmt.elif_body:
                        loop_body(body)
                    loop_body(stmt.else_body)
                elif isinstance(stmt, SchemaAttr):
                    add_name(stmt.name)

        def add_name(name: str):
            """Add the `name` into schema attr list"""
            if not name or not isinstance(name, str):
                return
            attr_list.append(name)

        loop_body(self.body)
        return attr_list


class RuleStmt(Stmt):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "RuleStmt"
        self.doc: str = ""
        self.name: str = ""
        self.parent_rules: List[Identifier] = []
        self.decorators: List[Decorator] = []
        self.checks: List[CheckExpr] = []
        self.name_node: Optional[Name] = None
        self.args: Optional[Arguments] = None
        self.for_host_name: Optional[Identifier] = None


class IfExpr(Expr):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "IfExpr"

        # body if cond else orelse
        self.body: Optional[Expr] = None  # self.body
        self.cond: Optional[Expr] = None  # self.cond
        self.orelse: Optional[Expr] = None  # self.orelse


class UnaryExpr(Expr):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "UnaryExpr"
        self.op: Optional[UnaryOp] = None
        self.operand: Optional[Expr] = None


class BinaryExpr(Expr):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "BinaryExpr"
        self.left: Optional[Expr] = None
        self.op: Optional[Union[BinOp, CmpOp]] = None
        self.right: Optional[Expr] = None

    def __str__(self):
        return (
            super().__str__()[:-1]
            + f" left: {self.left} op: {self.op} right: {self.right}>"
        )


class SelectorExpr(Expr):
    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
        ctx: ExprContext = ExprContext.LOAD,
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "SelectorExpr"
        self.value: Optional[Expr] = None
        self.attr: Optional[Identifier] = None
        self.ctx: Optional[ExprContext] = ctx
        self.has_question: bool = False


class CallExpr(Expr):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "CallExpr"
        self.func: Optional[Expr] = None
        self.args: List[Expr] = []
        self.keywords: List[Keyword] = []

    def __str__(self):
        return super().__str__()[:-1] + f" func: {self.func} args: {self.args}>"


class ParenExpr(Expr):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "ParenExpr"
        self.expr: Optional[Expr] = None


class QuantExpr(Expr):
    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
        ctx: ExprContext = ExprContext.LOAD,
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "QuantExpr"
        self.target: Optional[Expr] = None
        self.variables: List[Identifier] = []
        self.op: Optional[int] = None
        self.test: Optional[Expr] = None
        self.if_cond: Optional[Expr] = None
        self.ctx: ExprContext = ctx


class QuantOperation:
    ALL = 1
    ANY = 2
    FILTER = 3
    MAP = 4

    MAPPING = {
        "any": ANY,
        "all": ALL,
        "map": MAP,
        "filter": FILTER,
    }


class ListExpr(Expr):
    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
        ctx: ExprContext = ExprContext.LOAD,
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "ListExpr"
        self.elts: List[Expr] = []
        self.ctx: ExprContext = ctx

    def __str__(self):
        return super().__str__()[:-1] + f" values: {self.elts}>"


class ListIfItemExpr(Expr):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "ListIfItemExpr"
        self.if_cond: Optional[Expr] = None
        self.exprs: List[Expr] = []
        self.orelse: Optional[Expr] = None


class ListComp(Expr):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "ListComp"
        self.elt: Optional[Expr] = None
        self.generators: List[CompClause] = []


class StarredExpr(Expr):
    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
        ctx: ExprContext = ExprContext.LOAD,
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "StarredExpr"
        self.value: Optional[Expr] = None
        self.ctx: ExprContext = ctx


class DictComp(Expr):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "DictComp"
        self.key: Optional[Expr] = None
        self.value: Optional[Expr] = None
        self.operation: Optional[ConfigEntryOperation] = None
        self.generators: List[CompClause] = []


class ConfigIfEntryExpr(Expr):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "ConfigIfEntryExpr"
        self.if_cond: Optional[Expr] = None
        self.keys: List[Expr] = []
        self.values: List[Expr] = []
        self.operations: List[int] = []
        self.orelse: Optional[Expr] = None


class CompClause(Expr):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "CompClause"
        self.targets: List[Identifier] = []
        self.iter: Optional[Expr] = None
        self.ifs: List[Expr] = []


class SchemaExpr(Expr):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "SchemaExpr"
        self.name: Optional[Identifier] = None
        self.args: List[Expr] = []
        self.kwargs: List[Keyword] = []
        self.config: Optional[ConfigExpr] = None

    def __str__(self):
        return super().__str__()[:-1] + f" name: {self.name} config: {self.config}>"


class ConfigExpr(Expr):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "ConfigExpr"
        self.items: List[ConfigEntry] = []

    @property
    def keys(self) -> List[Expr]:
        return [item.key for item in self.items]

    @property
    def values(self) -> List[Expr]:
        return [item.value for item in self.items]

    @property
    def operations(self) -> List[int]:
        return [item.operation for item in self.items]

    def __str__(self):
        return super().__str__()[:-1] + f" keys: {self.keys} values: {self.values}>"


class ConfigEntryOperation:
    UNION = 0
    OVERRIDE = 1
    INSERT = 2
    UNIQUE = 3
    UNIFICATION = 4
    MAPPING = {
        LarkToken.L_COLON: UNION,
        LarkToken.L_ASSIGN: OVERRIDE,
        LarkToken.L_COMP_PLUS: INSERT,
    }

    @staticmethod
    def get_min():
        return ConfigEntryOperation.UNION

    @staticmethod
    def get_max():
        return ConfigEntryOperation.UNIFICATION


class ConfigEntry(AST):
    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
        key: Expr = None,
        value: Expr = None,
        operation: int = ConfigEntryOperation.UNION,
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "ConfigEntry"
        self.key: Expr = key
        self.value: Expr = value
        self.operation: int = operation
        self.insert_index: int = -1

    def __str__(self):
        return super().__str__()[:-1] + f" key: {self.key} value: {self.value}>"


class CheckExpr(Expr):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "CheckExpr"
        self.test: Optional[Expr] = None
        self.if_cond: Optional[Expr] = None
        self.msg: Optional[Expr] = None


class LambdaExpr(Expr):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "LambdaExpr"
        self.args: Optional[Arguments] = None
        self.return_type_str: Optional[str] = None
        self.return_type_node: Optional[Type] = None
        self.body: List[Stmt] = []


class Decorator(Expr):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "Decorator"
        self.name: Optional[Identifier] = None
        self.args: Optional[CallExpr] = None


class Subscript(Expr):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "Subscript"
        self.value: Optional[Expr] = None
        self.index: Optional[Expr] = None
        self.lower: Optional[Expr] = None
        self.upper: Optional[Expr] = None
        self.step: Optional[Expr] = None
        self.ctx: ExprContext = ExprContext.LOAD
        self.has_question: bool = False


class Keyword(Expr):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "Keyword"
        self.arg: Optional[Identifier] = None
        self.value: Optional[Expr] = None


class Arguments(Expr):
    def __init__(
        self, line: Optional[int] = None, column: Optional[int] = None
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "Arguments"
        self.args: List[Identifier] = []  # arg0, arg1, ...
        self.defaults: List[Expr] = []
        self.type_annotation_list: List[str] = []
        self.type_annotation_node_list: List[Type] = []

    def __str__(self):
        return super().__str__()[:-1] + f" args: {self.args}>"

    def GetArgName(self, i: int) -> str:
        if 0 <= i < len(self.args):
            return self.args[i].get_name()
        return ""

    def GetArgType(self, i: int) -> str:
        if 0 <= i < len(self.type_annotation_list):
            return self.type_annotation_list[i]
        return ""

    def SetArgType(self, i: int, tpe: str):
        if 0 <= i < len(self.type_annotation_list):
            self.type_annotation_list[i] = tpe

    def GetArgDefault(self, i: int) -> Optional[str]:
        if 0 <= i < len(self.defaults):
            return (
                typing.cast(StringLit, self.defaults[i]).value
                if self.defaults[i]
                else None
            )
        return None


class Compare(Expr):
    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "Compare"
        self.left: Optional[Expr] = None
        self.ops: List[CmpOp] = []
        self.comparators: List[Expr] = []


class Identifier(Expr):
    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
        names: List[str] = None,
        ctx: ExprContext = ExprContext.LOAD,
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "Identifier"
        self.names: List[str] = names if names else []
        self.pkgpath: str = ""
        self.ctx: ExprContext = ctx
        self.name_nodes: List[Name] = []

    def set_filename(self, filename: str) -> Expr:
        self.filename = filename
        return self

    def get_name(self, wrapper=True):
        return ".".join(self.names) if wrapper else self.names[-1]

    def get_first_name(self):
        return self.names[0]

    def set_ctx(self, ctx: ExprContext):
        self.ctx = ctx

    def __str__(self):
        return super().__str__()[:-1] + f" name: {self.get_name()}>"


class Literal(Expr):
    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
        value=None,
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "Literal"
        self.value = value

    def __str__(self):
        return super().__str__()[:-1] + f" value: {self.value}>"


class NumberLit(Literal):
    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
        value: Optional[int] = None,
    ) -> None:
        super().__init__(line, column, value)
        self._ast_type = "NumberLit"
        self.binary_suffix: Optional[str] = None


class NumberBinarySuffix:
    n = "n"
    u = "u"
    m = "m"
    k = "k"
    K = "K"
    M = "M"
    G = "G"
    T = "T"
    P = "P"
    Ki = "Ki"
    Mi = "Mi"
    Gi = "Gi"
    Ti = "Ti"
    Pi = "Pi"


class StringLit(Literal):
    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
        value: Optional[str] = None,
    ) -> None:
        super().__init__(line, column, value)
        self._ast_type = "StringLit"
        self.is_long_string: bool = False
        self.raw_value: Optional[str] = None


class NameConstantLit(Literal):
    """
    Name constant: True, False, None
    """

    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
        value: Optional[bool] = None,
    ) -> None:
        super().__init__(line, column, value)
        self._ast_type = "NameConstantLit"


class JoinedString(Expr):
    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "JoinedString"
        self.is_long_string: bool = False
        self.values: List[Union[StringLit, FormattedValue]] = []
        self.raw_value: Optional[str] = None


class FormattedValue(Expr):
    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
        value: Optional[Expr] = None,
        format_spec: str = None,
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "FormattedValue"
        self.is_long_string: bool = False
        self.value: Optional[Expr] = value
        self.format_spec: str = format_spec


class Comment(AST):
    def __init__(
        self,
        filename: Optional[str] = None,
        line: Optional[int] = None,
        column: Optional[int] = None,
        end_line: Optional[int] = None,
        end_column: Optional[int] = None,
        text: Optional[str] = None,
    ) -> None:
        super().__init__(line, column, end_line=end_line, end_column=end_column)
        self.filename = filename
        self._ast_type = "Comment"
        self.text: Optional[str] = text

    def __str__(self):
        return super().__str__()[:-1] + f" text: {self.text}>"


class CommentGroup(AST):
    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "CommentGroup"
        self.comments: List[Comment] = []


class Type(AST):
    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "Type"
        # type_element: schema_type | basic_type | compound_type | literal_type
        self.type_elements: List[
            Union[Identifier, BasicType, ListType, DictType, LiteralType]
        ] = []
        self.plain_type_str: str = ""


class BasicType(AST):
    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "BasicType"
        self.type_name: str = ""


class ListType(AST):
    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "ListType"
        self.inner_type: Optional[Type] = None
        self.plain_type_str: str = ""


class DictType(AST):
    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "DictType"
        self.key_type: Optional[Type] = None
        self.value_type: Optional[Type] = None
        self.plain_type_str: str = ""


class LiteralType(AST):
    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "LiteralType"
        self.plain_value: str = ""
        self.value_type: str = ""
        self.string_value: Optional[StringLit] = None
        self.number_value: Optional[NumberLit] = None


class Module(AST):
    """Module is an abstract syntax tree for a single KCL file."""

    def __init__(
        self,
        line: Optional[int] = None,
        column: Optional[int] = None,
        pkg: str = "",
        filename: str = "",
    ) -> None:
        super().__init__(line, column)
        self._ast_type = "Module"
        self.pkg: str = pkg
        self.filename: str = filename
        self.body: List[Stmt] = []

        self.doc: str = ""
        self.name = self.pkg  # {__main__} or same as {self.pkg}

        self.global_names: List[str] = []
        self.local_names: Dict[str, List[str]] = {}

        self.comments: List[
            Comment
        ] = []  # List of all comments in the source KCL module

    def __str__(self):
        return (
            super().__str__()[:-1]
            + f" pkg: {self.pkg} filename: {self.filename} body: {self.body}>"
        )

    def GetPkgpath(self, asname: str) -> str:
        for stmt in self.body or []:
            if isinstance(stmt, ImportStmt):
                import_spec = typing.cast(ImportStmt, stmt)
                if import_spec.asname == asname:
                    return import_spec.path
        return ""

    def GetImportList(self) -> List[ImportStmt]:
        import_list: List[ImportStmt] = []

        for stmt in self.body or []:
            if isinstance(stmt, ImportStmt):
                import_list.append(stmt)

        return import_list

    def GetSchemaList(self) -> List[SchemaStmt]:
        schema_list: List[SchemaStmt] = []

        for stmt in self.body or []:
            if isinstance(stmt, SchemaStmt):
                schema_list.append(stmt)

        return schema_list

    def GetSchemaAndRuleList(self) -> List[Union[SchemaStmt, RuleStmt]]:
        schema_rule_list: List[Union[SchemaStmt, RuleStmt]] = []

        for stmt in self.body or []:
            if isinstance(stmt, (SchemaStmt, RuleStmt)):
                schema_rule_list.append(stmt)

        return schema_rule_list

    def GetFileName(self, root: str = "") -> str:
        """# Construct the filename from the root path and relative file."""
        if root and self.relative_filename and self.relative_filename[0] == ".":
            filename = root + self.relative_filename[1:]
        else:
            filename = self.filename
        return filename

    def GetFirstExprInExprStmt(self) -> Expr:
        if self.body and len(self.body) > 0:
            if isinstance(self.body[0], ExprStmt) and len(self.body[0].exprs) > 0:
                return self.body[0].exprs[0]
        return None


class ASTFactory:
    @staticmethod
    def get_ast_module(node, pkg, filename, name) -> Module:
        check_utils.check_type_allow_none(filename, str, PosixPath)
        check_utils.check_allow_none(pkg, str)

        p = Module()
        p.line = 1
        p.column = 1
        p.pkg = pkg
        p.set_ast_position(node, filename)
        p.name = name
        return p

    @staticmethod
    def get_ast_configentry(key, value, operation, filename) -> ConfigEntry:
        check_utils.check_type_allow_none(key, Identifier, StringLit)
        check_utils.check_allow_none(value, Expr)
        check_utils.check_type_allow_none(filename, str, PosixPath)
        p = ConfigEntry(
            line=key.line if key else value.line,
            column=key.column if key else value.column,
            key=key,
            value=value,
            operation=operation,
        )
        p.filename = filename
        return p

    @staticmethod
    def get_ast_identifier(value: str) -> Identifier:
        check_utils.check_not_none(value, str)
        p = Identifier()
        p.names = value.split(".")
        return p

    @staticmethod
    def get_ast_literal(tpe: typing.Type[AST], line, column, value, filename):
        assert tpe
        check_utils.check_allow_none(line, int)
        check_utils.check_allow_none(column, int)
        check_utils.check_type_allow_none(filename, str, PosixPath)
        p = tpe(line, column, value)
        p.filename = filename
        return typing.cast(tpe, p)

    @staticmethod
    def get_ast(tpe: typing.Type[AST], node, filename):
        assert tpe
        check_utils.check_type_allow_none(filename, str, PosixPath)
        p = tpe().set_ast_position(node, filename)
        return typing.cast(tpe, p)

    @staticmethod
    def get_op(tpe: typing.Type[Enum], op_type: str):
        assert tpe
        check_utils.check_allow_none(op_type, str)
        p = tpe(op_type) if op_type else None
        return typing.cast(tpe, p)

    @staticmethod
    def get_ast_formatted_value(value, format_spec, filename):
        check_utils.check_type_allow_none(filename, str, PosixPath)
        check_utils.check_allow_none(value, Expr)
        check_utils.check_allow_none(format_spec, str)
        p = FormattedValue(value=value, format_spec=format_spec)
        p.set_ast_position(value, filename)
        return p


class Program:
    """Program is the AST collection of all files of the running KCL program."""

    MAIN_PKGPATH = "__main__"

    def __init__(
        self,
        *,
        root: str = "",
        main: str = "",
        pkgs: Dict[str, List[Module]] = None,
        cmd_args: List[CmdArgSpec] = None,
        cmd_overrides: List[CmdOverrideSpec] = None,
    ):
        self.root: str = root if root else ""
        self.main: str = main if main else ""
        self.pkgs: Dict[str, List[Module]] = pkgs if pkgs else {}

        self.cmd_args: List[CmdArgSpec] = cmd_args if cmd_args else []
        self.cmd_overrides: List[CmdOverrideSpec] = (
            cmd_overrides if cmd_overrides else []
        )

    def get_check_sum(self, root: str = "") -> str:
        """Get the AST program all file md5 sum"""
        check_sum = hashlib.md5()
        for modules in self.pkgs.values():
            for module in modules:
                if (
                    root
                    and module.relative_filename
                    and module.relative_filename[0] == "."
                ):
                    filename = root + module.relative_filename[1:]
                else:
                    filename = module.filename
                if os.path.isfile(filename):
                    # Encoding the filename into the checksum
                    check_sum.update(
                        (filename.replace(root, ".", 1) if root else filename).encode(
                            encoding="utf-8"
                        )
                    )
                    with open(filename, "rb") as f:
                        check_sum.update(f.read())
        return check_sum.hexdigest()

    def to_json(self, indent=4, sort_keys=False):
        return json.dumps(
            self,
            default=lambda o: o.value
            if type(o) in AST_ENUM_LIST.values()
            else o.__dict__,
            indent=indent,
            sort_keys=sort_keys,
        )
