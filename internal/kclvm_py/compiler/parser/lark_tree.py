#! /usr/bin/env python3

import os
import json
from typing import Union, List, Dict, Optional
from copy import deepcopy

from lark.exceptions import UnexpectedCharacters, UnexpectedToken
from lark.tree import Tree as LarkTree
from lark.lexer import Token as LarkToken

import kclvm.kcl.error as kcl_error
from kclvm.compiler.parser.lark_parser import GetKclLarkParser

TREE_TYPE = "tree"
TOKEN_TYPE = "token"


class Token:
    ASSIGN = "ASSIGN"  # "="
    COLON = "COLON"  # ":"
    SEMI_COLON = "SEMI_COLON"  # ";"
    COMMA = "COMMA"  # ","
    LEFT_PARENTHESES = "LEFT_PARENTHESES"  # "("
    RIGHT_PARENTHESES = "RIGHT_PARENTHESES"  # ")"
    LEFT_BRACKETS = "LEFT_BRACKETS"  # "["
    RIGHT_BRACKETS = "RIGHT_BRACKETS"  # "]"
    LEFT_BRACE = "LEFT_BRACE"  # "{"
    RIGHT_BRACE = "RIGHT_BRACE"  # "}"
    PLUS = "PLUS"  # "+"
    MINUS = "MINUS"  # "-"
    MULTIPLY = "MULTIPLY"  # "*"
    DIVIDE = "DIVIDE"  # "/"
    MOD = "MOD"  # "%"
    DOT = "DOT"  # "."
    AND = "AND"  # "&"
    OR = "OR"  # "|"
    XOR = "XOR"  # "^"
    NOT = "NOT"  # "~"
    LESS_THAN = "LESS_THAN"  # "<"
    GREATER_THAN = "GREATER_THAN"  # ">"
    EQUAL_TO = "EQUAL_TO"  # "=="
    NOT_EQUAL_TO = "NOT_EQUAL_TO"  # "!="
    GREATER_THAN_OR_EQUAL_TO = "GREATER_THAN_OR_EQUAL_TO"  # ">="
    LESS_THAN_OR_EQUAL_TO = "LESS_THAN_OR_EQUAL_TO"  # "<="
    DOUBLE_STAR = "DOUBLE_STAR"  # "**"
    DOUBLE_DIVIDE = "DOUBLE_DIVIDE"  # "//"
    SHIFT_LEFT = "SHIFT_LEFT"  # "<<"
    SHIFT_RIGHT = "SHIFT_RIGHT"  # ">>"

    COMP_PLUS = "COMP_PLUS"  # "+="
    COMP_MINUS = "COMP_MINUS"  # "-="
    COMP_MULTIPLY = "COMP_MULTIPLY"  # "*="
    COMP_DIVIDE = "COMP_DIVIDE"  # "/="
    COMP_MOD = "COMP_MOD"  # "%="
    COMP_AND = "COMP_AND"  # "&="
    COMP_OR = "COMP_OR"  # "|="
    COMP_XOR = "COMP_XOR"  # "^="
    COMP_NOT = "COMP_NOT"  # "~="
    COMP_DOUBLE_STAR = "COMP_DOUBLE_STAR"  # "**="
    COMP_DOUBLE_DIVIDE = "COMP_DOUBLE_DIVIDE"  # "//="
    COMP_SHIFT_LEFT = "COMP_SHIFT_LEFT"  # "<<="
    COMP_SHIFT_RIGHT = "COMP_SHIFT_RIGHT"  # ">>="

    IMPORT = "IMPORT"  # "import"
    AS = "AS"  # "as"
    DEF = "DEF"  # "def"
    LAMBDA = "LAMBDA"  # "lambda"
    SCHEMA = "SCHEMA"  # "schema"
    MIXIN = "MIXIN"  # "mixin"
    PROTOCOL = "PROTOCOL"  # "protocol"
    RELAXED = "RELAXED"  # "relaxed"
    CHECK = "CHECK"  # "check"
    INIT = "INIT"  # "init"
    TYPE = "TYPE"  # "type"
    FOR = "FOR"  # "for"
    ASSERT = "ASSERT"  # "assert"
    IF = "IF"  # "if"
    ELIF = "ELIF"  # "elif"
    ELSE = "ELSE"  # "else"
    L_OR = "L_OR"  # "or"
    L_AND = "L_AND"  # "and"
    L_NOT = "NOT"  # "not"
    L_L_NOT = "L_NOT"

    IN = "IN"  # "in"
    IS = "IS"  # "is"
    FINAL = "FINAL"  # "final"

    ALL = "ALL"
    ANY = "ANY"
    MAP = "MAP"
    FILTER = "FILTER"

    TRUE = "TRUE"
    FALSE = "FALSE"
    NONE = "NONE"

    NAME = "NAME"
    COMMENT = "COMMENT"
    NEWLINE = "NEWLINE"

    STRING = "STRING"
    LONG_STRING = "LONG_STRING"

    DEC_NUMBER = "DEC_NUMBER"
    HEX_NUMBER = "HEX_NUMBER"
    OCT_NUMBER = "OCT_NUMBER"
    BIN_NUMBER = "BIN_NUMBER"
    FLOAT_NUMBER = "FLOAT_NUMBER"
    IMAG_NUMBER = "IMAG_NUMBER"

    RIGHT_ARROW = "RIGHT_ARROW"

    @staticmethod
    def is_string(token: str):
        return token in [Token.STRING, Token.LONG_STRING]


class Tree:
    DICT_KEY = "dict_key"
    DICT_COMP = "dict_comp"
    LIST_COMP = "list_comp"
    DICT_EXPR = "dict_expr"
    LIST_EXPR = "list_expr"
    MULTI_EXPR = "multi_expr"
    DOUBLE_STAR_EXPR = "double_star_expr"
    CONFIG_EXPR = "config_expr"
    SUB_SCRIPT = "subscript"
    IDENTIFIER = "identifier"
    STR_CALL_EXPR = "str_call_expr"
    CALL_EXPR = "call_expr"
    CALL_SUFFIX = "call_suffix"
    SLICE_SUFFIX = "slice_suffix"
    COMP_ITER = "comp_iter"
    COMP_FOR = "comp_for"
    COMP_IF = "comp_if"
    COMP_CLAUSE = "comp_clause"
    TEST = "test"
    OR_TEST = "or_test"
    SIMPLE_EXPR = "simple_expr"
    EXPR = "expr"
    PRIMARY_EXPR = "primary_expr"
    STMT = "stmt"
    IF_STMT = "if_stmt"
    SCHEMA_MEMBER_STMT = ("schema_member_stmt",)
    SCHEMA_BODY = "schema_body"
    SCHEMA_ARGUMENTS = "schema_arguments"
    MIXINS = ("mixins",)
    TYPE_ALIAS_STMT = "type_alias_stmt"
    SCHEMA_STMT = "schema_stmt"
    SIMPLE_STMT = "simple_stmt"
    DOC_STMT = "doc_stmt"
    COMPOUND_STMT = "compound_stmt"
    SMALL_STMT = "small_stmt"
    IMPORT_STMT = "import_stmt"
    ASSIGN_STMT = "assign_stmt"
    ASSERT_STMT = "assert_stmt"
    EXPR_STMT = "expr_stmt"
    MEMBER_STMT = "member_stmt"
    TYPE = "type"
    EXECUTION_BLOCK = "execution_block"
    STRING_DOT_NAME = "string_dot_name"

    SELECTOR_EXPR = "selector_expr"
    SELECTOR_SUFFIX = "selector_suffix"
    LIST_SELECTOR_SUFFIX = "list_selector_suffix"
    DICT_SELECTOR_SUFFIX = "dict_selector_suffix"


BRACKETS_TOKENS = [
    Token.LEFT_PARENTHESES,  # "("
    Token.RIGHT_PARENTHESES,  # ")"
    Token.LEFT_BRACKETS,  # "["
    Token.RIGHT_BRACKETS,  # "]"
    Token.LEFT_BRACE,  # "{"
    Token.RIGHT_BRACE,  # "}"
]
OPERATOR_TOKENS = {
    Token.ASSIGN,  # "="
    Token.PLUS,  # "+"
    Token.MINUS,  # "-"
    Token.MULTIPLY,  # "*"
    Token.DIVIDE,  # "/"
    Token.MOD,  # "%"
    Token.AND,  # "&"
    Token.OR,  # "|"
    Token.XOR,  # "^"
    Token.NOT,  # "~"
    Token.LESS_THAN,  # "<"
    Token.GREATER_THAN,  # ">"
    Token.EQUAL_TO,  # "=="
    Token.NOT_EQUAL_TO,  # "!="
    Token.GREATER_THAN_OR_EQUAL_TO,  # ">="
    Token.LESS_THAN_OR_EQUAL_TO,  # "<="
    Token.DOUBLE_STAR,  # "**"
    Token.DOUBLE_DIVIDE,  # "//"
    Token.SHIFT_LEFT,  # "<<"
    Token.SHIFT_RIGHT,  # ">>"
    Token.COMP_PLUS,  # "+="
    Token.COMP_MINUS,  # "-="
    Token.COMP_MULTIPLY,  # "*="
    Token.COMP_DIVIDE,  # "/="
    Token.COMP_MOD,  # "%="
    Token.COMP_AND,  # "&="
    Token.COMP_OR,  # "|="
    Token.COMP_XOR,  # "^="
    Token.COMP_NOT,  # "~="
    Token.COMP_DOUBLE_STAR,  # "**="
    Token.COMP_DOUBLE_DIVIDE,  # "//="
    Token.COMP_SHIFT_LEFT,  # "<<="
    Token.COMP_SHIFT_RIGHT,  # ">>="
}
SEPERATOR_TOKENS = {
    Token.COLON,  # ":"
    Token.SEMI_COLON,  # ";"
    Token.COMMA,  # ","
}
FUNCTION_EXPRS = {
    Tree.CALL_EXPR,
    Tree.STR_CALL_EXPR,
}
_WALK_FUNCTION_PREFIX = "walk_"

AstType = Dict[str, Union[str, List]]


class TreeWalker:
    """
    The TreeWalk class can be used as a superclass in order
    to walk an AST or similar tree.

    This class is meant to be subclassed, with the subclass adding walker
    methods.

    Per default the walker functions for the nodes are ``'walk_'`` +
    class name of the node.  So a `expr_stmt` node visit function would
    be `walk_expr_stmt`.  This behavior can be changed by overriding
    the `walk` method.  If no walker function exists for a node
    (return value `None`) the `generic_walker` walker is used instead.
    """

    def __init__(self) -> None:
        pass

    def walk(self, node: AstType) -> None:
        """Visit a node."""
        name = node["name"]
        method = _WALK_FUNCTION_PREFIX + name
        walker = getattr(self, method, self.generic_walk)
        walker(node)

    def generic_walk(self, node: AstType):
        """Called if no explicit walker function exists for a node."""
        children_key = "children"
        if children_key in node:
            for n in node[children_key]:
                self.walk(n)
        else:
            self.walk(node)

    def walk_nodes(self, *nodes: Union[AstType, str]) -> None:
        """Write nodes"""
        if not nodes:
            return
        for node in nodes:
            self.walk_node(node)

    def walk_node(self, node: Union[AstType, str]) -> None:
        """Write node"""
        self.walk(node)

    def get(self, node: AstType, name: str) -> Optional[AstType]:
        """
        Get children from 'node' named 'name'
        """
        if not node or "children" not in node:
            return None
        for i, n in enumerate(node["children"]):
            if n["name"] == name:
                node["children"] = node["children"][i + 1 :]
                return n
        return None

    def has(self, node: AstType, name: str) -> bool:
        """
        Whether 'node' has the children named 'name'
        """
        return name in [n["name"] for n in node["children"]]

    def get_value(self, node: AstType, default: Optional[str] = None) -> Optional[str]:
        """
        Get tree node value recursively
        """
        if not node:
            return default
        if node["type"] == TOKEN_TYPE:
            return node.get("value", default)
        elif node["type"] == TREE_TYPE:
            return "".join(self.get_value(n) for n in node["children"])
        return default

    def get_children_value(
        self, node: AstType, name: Optional[str] = None, default: Optional[str] = None
    ) -> Optional[str]:
        """
        Get children value recursively from 'node' named 'name'
        """
        return self.get_value(self.get(node, name), default) if name else default

    def get_internal(
        self, node: AstType, begin_name: str, end_name: str
    ) -> Optional[List[AstType]]:
        """
        Get children from 'begin_name' to 'end_name'
        """
        begin_index = -1
        end_index = -1
        for i, n in enumerate(node["children"]):
            if n["name"] == begin_name and begin_index == -1:
                begin_index = i
            if n["name"] == end_name:
                end_index = i
        if end_index >= begin_index:
            result = deepcopy(node["children"][begin_index : end_index + 1])
            node["children"] = node["children"][end_index + 1 :]
            return result
        return None


def build_children(t: LarkTree) -> List[AstType]:
    """
    Build json ast children of tree node, children type can be token or tree
    """
    return list(map(lambda c: walk_lark_tree(c), t.children))


def build_token_content(t: LarkToken) -> AstType:
    """
    Build json ast token node
    """
    return {
        "type": TOKEN_TYPE,
        "name": t.type,
        "value": t.value,
        "line": t.line,
        "column": t.column,
    }


def build_tree_content(t: LarkTree) -> AstType:
    """
    Build json ast tree node
    """
    return {
        "type": TREE_TYPE,
        "name": t.data,
        "children": build_children(t),
    }


def walk_lark_tree(t: Union[LarkTree, LarkToken]) -> AstType:
    """
    Traverse the lark tree and return python dict
    """
    if isinstance(t, LarkTree):
        return build_tree_content(t)
    elif isinstance(t, LarkToken):
        return build_token_content(t)
    else:
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.CompileError_TYPE,
            arg_msg="Unknown node type: {}".format(type(t)),
        )


def get_lark_tree_from_expr(
    source: str, to_json_str: bool = True
) -> Union[str, AstType]:
    try:
        # Get lark parse tree
        parse_tree = GetKclLarkParser().parse(source + "\n")
        # Convert python dict to standard json
        ast = walk_lark_tree(parse_tree)
        return (json.dumps(ast) + "\n") if to_json_str else ast
    except (
        UnexpectedCharacters,
        UnexpectedToken,
    ) as err:
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.InvalidSyntax_TYPE,
            file_msgs=[kcl_error.ErrFileMsg(line_no=err.line, col_no=err.column)],
        )
    except Exception as err:
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.CompileError_TYPE, arg_msg=str(err)
        )


def get_lark_tree_from_file(filename: str, to_json_str: bool = True) -> str:
    """
    Get kcl json ast from .k file
    """
    with open(filename, "r", encoding="utf-8") as pyfile:
        source = pyfile.read()
    return get_lark_tree_from_expr(source, to_json_str)


def get_lark_tree_from_path(path: str) -> List[str]:
    """
    Get kcl json ast from kmodule package
    """
    return [get_lark_tree_from_file(file) for file in sorted(os.listdir(path))]
