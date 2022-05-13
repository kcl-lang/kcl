# Copyright 2021 The KCL Authors. All rights reserved.

import re
import pathlib

from io import StringIO
from typing import List, Union

import kclvm.kcl.error as kcl_error
import kclvm.internal.log as klog

from kclvm.compiler.parser.lark_tree import get_lark_tree_from_expr
from kclvm.compiler.parser.lark_tree import (
    Token,
    Tree,
    AstType,
    TreeWalker,
    TOKEN_TYPE,
    TREE_TYPE,
    OPERATOR_TOKENS,
)

_INVALID_NEWLINE_STRING_MSG = "invalid NEWLINE token string value {}"
_INLINE_COMMENT_REGEX = "#[^\n]*\n"
_INLINE_COMMENT_WITH_MULTILINE_REGEX = "#[^\n]*[\n\t ]*"
_NODE_WITH_NEWLINE_EXPRS = [
    Tree.CONFIG_EXPR,
    Tree.DICT_COMP,
    Tree.DICT_EXPR,
    Tree.LIST_COMP,
    Tree.LIST_EXPR,
    Tree.COMP_CLAUSE,
    Tree.SIMPLE_STMT,
    Tree.SCHEMA_MEMBER_STMT,
    Tree.MIXINS,
]
SEPARATOR_TOKEN = " "
EMPTY_TOKEN = ""
ENDLINE_TOKEN = "\n"
COMMENT_START_TOKEN = "#"


class TextAdapterWalker(TreeWalker):
    """
    Walker adapted for text processing, can be used as a semantic-independent
    walker super class such as formatter and linter
    """

    def __init__(self):
        super().__init__()
        self.printer = StringIO()  # Printer used to write expressions and tokens
        self.indent_level: int = 0  # Now indent level
        self.indent_queue = [0]  # Indent queue
        self.indent_space_count: int = 4  # Default indent space count

    def write(self, text: str) -> None:
        """Append a piece of text to the current line."""
        self.printer.write(text)

    def write_token_separator(self) -> None:
        """Print the separator between tokens."""
        self.write(SEPARATOR_TOKEN)

    def walk_node(self, node: Union[AstType, str]) -> None:
        """Write node"""
        if isinstance(node, str):
            self.write(node)
        else:
            self.walk(node)

    def fill(self, text: str = "") -> None:
        """Append a piece of text to the current line."""
        self.printer.write(ENDLINE_TOKEN)
        self.write_indent()
        self.printer.write(text)

    def write_indent(self) -> None:
        """Append indent white space according indent level"""
        self.printer.write(
            self.indent_space_count * self.indent_level * SEPARATOR_TOKEN
        )

    def count_blank_line(self, text: str) -> int:
        """Blank line count in a NEWLINE token"""
        return re.sub(_INLINE_COMMENT_REGEX, EMPTY_TOKEN, text).count(ENDLINE_TOKEN) - 1

    def count_indent(self, text: str) -> int:
        """
        Count the indent by number of white spaces for a leading text
        e.g. NEWLINE "\n    " its indent count is 4
        e.g. NEWLINE "\n # inline comment\n   " its indent count is 3
        """
        if ENDLINE_TOKEN not in text:
            return 0
        line = text.split(ENDLINE_TOKEN)[-1]
        temptext = line.replace(SEPARATOR_TOKEN, EMPTY_TOKEN)
        count = len(line) if len(temptext) == 0 else 0
        if count not in self.indent_queue:
            # Indent increase
            self.indent_queue.append(count)
        else:
            # Indent decrease
            idx = self.indent_queue.index(count)
            del self.indent_queue[idx + 1 :]
        idx = self.indent_queue.index(count)
        self.indent_level = idx
        return count

    def generic_walk(self, node: AstType) -> None:
        """Called if no explicit walker function exists for a node."""
        if node["type"] == TOKEN_TYPE:
            self.walk_pre_token(node)
            self.walk_token(node)
            self.walk_post_token(node)
        elif node["type"] == TREE_TYPE:
            for n in node["children"]:
                self.walk(n)
        else:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.CompileError_TYPE,
                arg_msg="Unknown format node type",
            )

    def walk_pre_token(self, node: AstType) -> None:
        """Deal after token"""
        pass

    def walk_post_token(self, node: AstType) -> None:
        """Deal after token"""
        pass

    def walk_token(self, node: AstType) -> None:
        """AST: token"""
        pass


class Formatter(TextAdapterWalker):
    def __init__(self) -> None:
        super().__init__()
        self.last_token: str = EMPTY_TOKEN  # Last token name
        self.last_token_value: str = EMPTY_TOKEN  # Last token value
        self.single_comment_spaces: int = 2  # Inline comment after its expressions
        self.max_blank_line: int = 1  # Maximum number of blank lines
        self._is_in_arguments = False  # Mark is in arguments
        self._is_in_collection_if = False  # Mark is in collection if
        self._is_colon_without_space = False  # Mark is in expr with colon except space

    # Base

    def generic_walk(self, node: AstType) -> None:
        """Formatter special walk"""
        if node["name"] in _NODE_WITH_NEWLINE_EXPRS:
            self.write_node_with_newline(node)
        else:
            super().generic_walk(node)

    # Tokens

    def add_blank_line(self) -> None:
        """Blank line before and after schema_stmt"""
        if (
            self.last_token == Token.NEWLINE
            and self.count_blank_line(self.last_token_value) < self.max_blank_line
        ):
            self.write_token(ENDLINE_TOKEN)
            self.last_token = Token.NEWLINE
            self.last_token_value = ENDLINE_TOKEN * 2

    def split_newline_value(self, value: str) -> List[str]:
        """Split a NEWLINE token value into newline parts and inline comment parts

        Input: "\n \n # comment \n # comment \n\n # comment \n"
        Output: ["\n \n ", "# comment ", "\n ", "# comment ", "\n\n ", "# comment ", "\n"]
        """
        if not value:
            return []
        parts = []
        # Mark containing COMMENT token
        index = value.find(COMMENT_START_TOKEN)
        if index == -1:
            return [value]  # Single NEWLINE without COMMENT
        elif index > 0:
            parts.append(value[:index])  # Add first NEWLINE token
        inline_comments = re.findall(_INLINE_COMMENT_WITH_MULTILINE_REGEX, value)
        for comment in inline_comments:
            index = comment.find(ENDLINE_TOKEN)
            if index == -1:
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.CompileError_TYPE,
                    arg_msg=_INVALID_NEWLINE_STRING_MSG.format(comment),
                )
            parts.append(comment[:index])  # Add COMMENT token
            parts.append(comment[index:])  # Add NEWLINE token
        if len(parts) > 1 and ENDLINE_TOKEN not in parts[-1]:
            # Last part is not NEWLINE, raise an error
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.CompileError_TYPE,
                arg_msg=_INVALID_NEWLINE_STRING_MSG.format(value),
            )
        return parts

    def write_newline(self, value: str) -> None:
        """
        Write newline except inline comment
        """
        blank_line_count = self.count_blank_line(value)
        value = value.replace(SEPARATOR_TOKEN, "")
        newline_count = 0
        for i, c in enumerate(value):
            if c == ENDLINE_TOKEN:
                newline_count += 1
                # Consecutive blank lines
                if newline_count < blank_line_count:
                    pass
                # If there is a blank line, keep it
                elif self.count_blank_line(value[i + 1 :]):
                    self.fill()
                # First NEWLINE in last expr
                else:
                    self.write(ENDLINE_TOKEN)

    def walk_newline(self, token: AstType) -> None:
        """
        Format NEWLINE token contains '#', '\n' and ' '
        """
        # Record indent level and Windows line break handling and 1 tab <-> 4 spaces
        value = token["value"].replace("\r", ENDLINE_TOKEN).replace("\t", "    ")
        # Remove start blank lines except comments and first token is NEWLINE
        if self.last_token == "":
            # Get all inline comments
            self.write("".join(re.findall(_INLINE_COMMENT_REGEX, value)))
            return
        # Record indent
        self.count_indent(value)
        parts = self.split_newline_value(value)
        for part in parts:
            if part.startswith(COMMENT_START_TOKEN):
                self.write(part)
            else:
                self.write_newline(part)

    def write_string(self, value: str) -> None:
        """Print KCL string with prefix"""
        quota = False
        for c in value:
            # The string prefix is uniformly lowercase
            if not quota:
                c = c.lower()
            # String start quotation marks
            if c == '"' or c == "'":
                quota = True
            self.write(c)

    def write_token(self, node: Union[str, AstType]) -> None:
        """Write token node or token string"""
        if not node:
            return
        if isinstance(node, str):
            self.write(node)
        else:
            self.walk_token(node)

    def walk_post_token(self, token: AstType) -> None:
        """Deal after token"""
        name, value = token["name"], token["value"]
        if name == Token.ASSIGN and self._is_in_arguments:
            # Do not write space between '=' in kwargs
            pass
        elif (
            name == Token.COLON
            and self._is_colon_without_space
            and not self._is_in_arguments
        ):
            # Do not write space after colon ':' e.g. check: and schema Name:
            pass
        elif (
            name in [Token.PLUS, Token.MINUS, Token.NOT]
            and self.last_token in OPERATOR_TOKENS
        ):
            # Do not write space between unary operator
            pass
        elif name in OPERATOR_TOKENS:
            self.write_token_separator()
        elif name == Token.FOR and self.last_token != Token.NEWLINE:
            self.write_token_separator()
        elif name in [Token.IF, Token.ELIF] and self._is_in_collection_if:
            self.write_token_separator()
        # Write space after , and : except NEWLINE behind it and write space behind keyword
        elif name in [
            Token.IMPORT,
            Token.LAMBDA,
            Token.SCHEMA,
            Token.RELAXED,
            Token.ASSERT,
            Token.FINAL,
            Token.IN,
            Token.AS,
            Token.IS,
            Token.MIXIN,
            Token.PROTOCOL,
            Token.TYPE,
            Token.COMMA,
            Token.COLON,
            Token.L_AND,
            Token.L_OR,
            Token.L_L_NOT,
            Token.SEMI_COLON,
            Token.ALL,
            Token.ANY,
            Token.MAP,
            Token.FILTER,
        ]:
            self.write_token_separator()
        self.last_token, self.last_token_value = name, value

    def walk_pre_token(self, token: AstType) -> None:
        """Deal before token"""
        name = token["name"]
        value = token["value"]
        if name == Token.ASSIGN and self._is_in_arguments:
            # Do not write space between '=' in kwargs
            pass
        elif (
            name in [Token.PLUS, Token.MINUS, Token.NOT]
            and self.last_token in OPERATOR_TOKENS
        ):
            # Do not write space between unary operator
            pass
        elif name in OPERATOR_TOKENS:
            self.write_token_separator()
        elif name == Token.FOR and self.last_token != Token.NEWLINE:
            self.write_token_separator()
        elif name in [Token.IN, Token.AS, Token.IS, Token.L_AND, Token.L_OR]:
            self.write_token_separator()
        elif (
            name == Token.NEWLINE
            and value.startswith(COMMENT_START_TOKEN)
            and self.last_token != ""
        ):
            # Two spaces between expr and inline comment #
            self.write(SEPARATOR_TOKEN * self.single_comment_spaces)

    def walk_token(self, token: AstType) -> None:
        """AST: token node"""
        name = token["name"]
        value = token["value"]
        if name == Token.NEWLINE:
            self.walk_newline(token)
        elif Token.is_string(name):
            self.write_string(value)
        else:
            self.write_token(value)

    # Exprs

    def write_expr(self, node: AstType) -> None:
        """Write expr"""
        if node:
            self.walk_nodes(*node["children"])

    def write_arguments(self, nodes: List[AstType]) -> None:
        """Arguments in call_expr, str_expr and schema_expr"""
        self._is_in_arguments = True
        self.walk_nodes(*nodes)
        self._is_in_arguments = False

    def walk_call_suffix(self, node: AstType) -> None:
        """AST: call_suffix

        call_suffix: LEFT_PARENTHESES [arguments [COMMA]] RIGHT_PARENTHESES
        """
        self.write_arguments(node["children"])

    def walk_schema_expr(self, node: AstType) -> None:
        """AST: schema_expr

        schema_expr: identifier (LEFT_PARENTHESES [arguments] RIGHT_PARENTHESES)? dict_expr
        """
        self.write_expr(self.get(node, Tree.IDENTIFIER))
        self.write_arguments(
            self.get_internal(node, Token.LEFT_PARENTHESES, Token.RIGHT_PARENTHESES)
        )
        # Write space between 'SchemaName' and '{}' in schema_expr
        self.write_token_separator()
        self.write_expr(self.get(node, Tree.CONFIG_EXPR))

    def walk_star_expr(self, node: AstType) -> None:
        """AST: star_expr

        star_expr: MULTIPLY primary_expr
        """
        # Do not write space between * in list iter and var
        if self.get(node, Token.MULTIPLY):
            self.write("*")
        self.write_expr(self.get(node, Tree.PRIMARY_EXPR))

    def walk_double_star_expr(self, node: AstType) -> None:
        """AST: double_star_expr

        double_star_expr: DOUBLE_STAR primary_expr
        """
        # Do not write space between ** in dict iter and var
        if self.get(node, Token.DOUBLE_STAR):
            self.write("**")
        self.write_expr(self.get(node, Tree.PRIMARY_EXPR))

    def walk_lambda_expr(self, node: AstType) -> None:
        """AST: if_expr

        lambda_expr: LAMBDA [schema_arguments] [RIGHT_ARROW type] LEFT_BRACE [expr_stmt | NEWLINE _INDENT schema_init_stmt+ _DEDENT] RIGHT_BRACE
        """
        for n in node["children"]:
            self.walk_node(n)
            if n["name"] in [Tree.SCHEMA_ARGUMENTS, Token.RIGHT_ARROW, Tree.TYPE]:
                self.write_token_separator()

    def walk_if_expr(self, node: AstType) -> None:
        """AST: if_expr

        if_expr: or_test IF or_test ELSE test
        """
        self.write_expr(self.get(node, Tree.SIMPLE_EXPR))
        while self.get(node, Token.IF):
            self.write_token_separator()
            self.write_token("if")
            self.write_token_separator()
            self.write_expr(self.get(node, Tree.SIMPLE_EXPR))
            self.write_token_separator()
            self.write_token("else")
            self.write_token_separator()
            self.write_expr(self.get(node, Tree.TEST))

    def walk_comp_clause(self, node: AstType) -> None:
        """AST: comp_clause

        comp_clause: FOR loop_variables [COMMA] IN or_test [NEWLINE] [IF test [NEWLINE]]
        """
        for n in node["children"]:
            if n["name"] == Token.IF and self.last_token != Token.NEWLINE:
                self.write_token_separator()
            self.walk_node(n)
            if n["name"] == Token.IF:
                self.write_token_separator()

    def walk_quant_expr(self, node: AstType) -> None:
        """AST: quant_expr

        quant_expr: quant_op [ identifier COMMA ] identifier IN quant_target LEFT_BRACE (simple_expr [IF simple_expr] | NEWLINE _INDENT simple_expr [IF simple_expr] NEWLINE _DEDENT)? RIGHT_BRACE
        """
        for n in node["children"]:
            if n["name"] == Token.IF and self.last_token != Token.NEWLINE:
                self.write_token_separator()
            if n["name"] == Token.LEFT_BRACE:
                self.write_token_separator()
            self.walk_node(n)
            if n["name"] == Token.IF:
                self.write_token_separator()

    def write_node_with_newline(self, node: AstType) -> None:
        """
        Write node with , ; and NEWLINE
        such as list_comp and dict_comp.
        """
        children = node["children"]
        for i, n in enumerate(children):
            if (
                n["name"] in [Token.COMMA, Token.SEMI_COLON]
                and i < len(children) - 1
                and children[i + 1]["name"] == Token.NEWLINE
            ):
                self.write(n["value"])
            else:
                self.walk(n)

    def write_colon_without_space(self, node: AstType) -> None:
        """
        Write expr with colon : without space
        """
        self._is_colon_without_space = True
        self.write_expr(node)
        self._is_colon_without_space = False

    def walk_mixins(self, node: AstType) -> None:
        """AST: mixins

        mixins: identifier (COMMA (NEWLINE mixins | identifier))*
        """
        self.write_node_with_newline(node)

    def walk_list_items(self, node: AstType) -> None:
        """AST: list_items

        list_items: list_item ((COMMA [NEWLINE] | NEWLINE) list_item)* [COMMA] [NEWLINE]
        """
        self.write_node_with_newline(node)

    def walk_entries(self, node: AstType) -> None:
        """AST: entries

        entries: entry ((COMMA [NEWLINE] | NEWLINE) entry)* [COMMA] [NEWLINE]
        """
        self.write_node_with_newline(node)

    def walk_config_entries(self, node: AstType) -> None:
        """AST: config_entries

        entries: entry ((COMMA [NEWLINE] | NEWLINE) entry)* [COMMA] [NEWLINE]
        """
        self.write_node_with_newline(node)

    def walk_subscript_suffix(self, node: AstType) -> None:
        """AST: subscript_suffix

        subscript_suffix: LEFT_BRACKETS (test | [test] COLON [test] [COLON [test]]) RIGHT_BRACKETS
        """
        self.write_colon_without_space(node)

    def walk_slice_suffix(self, node: AstType) -> None:
        """AST: slice_suffix

        slice_suffix: LEFT_BRACKETS (test | [test] COLON [test] [COLON [test]]) RIGHT_BRACKETS
        """
        self.write_colon_without_space(node)

    def walk_bin_op(self, node: AstType) -> None:
        """AST: comp_op"""
        children = node["children"]
        for i, n in enumerate(children):
            # IN | L_NOT IN | IS | IS L_NOT | L_NOT
            if n["name"] == Token.IS:
                self.write_token_separator()
                self.write(n["value"])
                if i + 1 < len(children) and children[i + 1]["name"] != Token.L_L_NOT:
                    self.write_token_separator()
            elif n["name"] == Token.L_L_NOT:
                self.write_token_separator()
                if i == 0:
                    self.write(n["value"])
                    if i + 1 < len(children) and children[i + 1]["name"] != Token.IN:
                        self.write_token_separator()
                if i == 1:
                    self.write("not")
                    self.write_token_separator()
            else:
                self.walk(n)

    def walk_not_test(self, node: AstType) -> None:
        """AST: not_test"""
        if self.get(node, Token.L_NOT):
            self.write_token("not")
            self.write_token_separator()
        self.write_expr(node)

    def walk_check_block(self, node: AstType) -> None:
        """AST: check_block

        check_block: CHECK COLON NEWLINE _INDENT check_expr+ _DEDENT
        """
        self.write_token(self.get(node, Token.CHECK))
        self.write_token(self.get(node, Token.COLON))
        self.write_expr(node)

    def walk_check_expr(self, node: AstType) -> None:
        """AST: check_expr

        check_expr: simple_expr [IF simple_expr] [COMMA primary_expr] NEWLINE
        """
        self.write_expr(self.get(node, Tree.SIMPLE_EXPR))
        if self.get(node, Token.IF):
            self.write_token_separator()
            self.write_token("if")
            self.write_token_separator()
            self.write_expr(self.get(node, Tree.SIMPLE_EXPR))
        for n in node["children"]:
            self.walk(n)

    def walk_dict_type(self, node: AstType) -> None:
        """AST: dict_type"""
        self.write_colon_without_space(node)

    # Statements

    def write_stmt(self, node: AstType) -> None:
        """Write stmt"""
        self.write_token(self.get(node, Token.NEWLINE))
        self.walk_nodes(*node["children"])

    def write_condition_and_body(self, node: AstType) -> None:
        """Write if and elif condition and body"""
        self.write_token_separator()
        self.write_expr(self.get(node, Tree.TEST))
        self.write_token(":")
        self.write_stmt(self.get(node, Tree.EXECUTION_BLOCK))

    def walk_start(self, node: AstType) -> None:
        """node: start
        start: (NEWLINE | statement)*
        statement: simple_stmt | compound_stmt
        simple_stmt: (assign_stmt | unification_stmt | expr_stmt | assert_stmt | import_stmt | type_alias_stmt) NEWLINE
        """
        last_stmt_is_import = False
        for n in node["children"]:
            if n["name"] != Token.NEWLINE:
                stmt_name = n["children"][0]["children"][0]["name"]
                # Add a blank line after consecutive import statements
                if last_stmt_is_import and stmt_name != Tree.IMPORT_STMT:
                    self.add_blank_line()
                last_stmt_is_import = stmt_name == Tree.IMPORT_STMT
            self.walk_node(n)

    def walk_if_stmt(self, node: AstType) -> None:
        """AST: if_stmt"""
        self.write_token("if")
        self.write_condition_and_body(node)
        while self.get(node, Token.ELIF):
            self.write_token("elif")
            self.write_condition_and_body(node)
        if self.get(node, Token.ELSE):
            self.write_token("else:")
            self.write_stmt(self.get(node, Tree.EXECUTION_BLOCK))

    def walk_assert_stmt(self, node: AstType) -> None:
        """Syntax
        assert_stmt: ASSERT simple_expr (IF simple_expr)? (COMMA test)?
        """
        self.write_token("assert")
        self.write_token_separator()
        self.write_expr(self.get(node, Tree.SIMPLE_EXPR))
        if self.get(node, Token.IF):
            self.write_token_separator()
            self.write_token("if")
            self.write_token_separator()
            self.write_expr(self.get(node, Tree.SIMPLE_EXPR))
        test_node = self.get(node, Tree.TEST)
        if test_node:
            self.write_token(", ")
            self.write_expr(test_node)

    def walk_schema_stmt(self, node: AstType) -> None:
        """AST: schema_stmt

        schema_stmt: [decorators] (SCHEMA|MIXIN|PROTOCOL) [RELAXED] NAME [LEFT_BRACKETS [schema_arguments] RIGHT_BRACKETS] [LEFT_PARENTHESES identifier (COMMA identifier)* RIGHT_PARENTHESES] COLON NEWLINE [schema_body]
        schema_body: _INDENT (string NEWLINE)* [mixin_stmt] (schema_attribute_stmt|schema_init_stmt)* [check_block] _DEDENT
        schema_attribute_stmt: attribute_stmt NEWLINE
        attribute_stmt: [decorators] (FINAL)? NAME COLON type [(ASSIGN|COMP_OR) test]
        schema_init_stmt: if_simple_stmt | if_stmt
        """

        self.add_blank_line()
        self._is_colon_without_space = True
        keywords = [Token.SCHEMA, Token.MIXIN, Token.PROTOCOL]
        for keyword in keywords:
            if self.has(node, keyword):
                self.walk_nodes(*self.get_internal(node, keyword, Token.COLON))
                break
        self._is_colon_without_space = False
        self.write_expr(node)
        self.add_blank_line()

    def walk_schema_argument(self, node: AstType):
        """AST: schema_argument

        schema_argument: NAME [COLON type] [ASSIGN test]
        """
        nodes = node["children"]
        if self.has(node, Token.COLON):
            self.walk_nodes(*nodes)
        else:
            self.write_arguments(nodes)

    def walk_if_item(self, node: AstType):
        """Syntax
        if_item: IF test COLON if_item_exec_block (ELIF test COLON if_item_exec_block)* (ELSE COLON if_item_exec_block)?
        """
        self._is_in_collection_if = True
        self.write_colon_without_space(node)
        self._is_in_collection_if = False

    def walk_if_entry(self, node: AstType):
        """Syntax
        if_entry: IF test COLON if_entry_exec_block (ELIF test COLON if_entry_exec_block)* (ELSE COLON if_entry_exec_block)?
        """
        self._is_in_collection_if = True
        for n in node["children"]:
            if n["name"] == Token.COLON:
                self.write(":")
            else:
                self.walk_node(n)
        self._is_in_collection_if = False

    # User interfaces

    def fmt_ast(self, ast: AstType) -> str:
        """Format main function and return the format string buffer"""
        if not isinstance(ast, dict):
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.CompileError_TYPE, arg_msg="Invalid ast type"
            )
        self.walk_node(ast)
        # Append file last blank line
        if self.last_token_value == ENDLINE_TOKEN:
            self.write(ENDLINE_TOKEN)
        self.printer.seek(0)
        return self.printer.read()[:-1]


def _get_kcl_files(file_or_dir: pathlib.Path, recursively=False):
    """
    Get files in dir
    """
    kcl_file_selector = "*.k"
    return (
        file_or_dir.rglob(kcl_file_selector)
        if recursively
        else file_or_dir.glob(kcl_file_selector)
    )


def kcl_ast_to_fmt_file(json_ast: AstType) -> str:
    """
    Update a kcl ast to a fmt file
    """
    return Formatter().fmt_ast(json_ast)


def kcl_fmt_source(source: str) -> (str, bool):
    """
    Format kcl code string and return formatted string
    """
    formatted_code = Formatter().fmt_ast(get_lark_tree_from_expr(source, False))
    return formatted_code, source != formatted_code


def kcl_fmt_dir(file_or_dir: pathlib.Path, _recursively=False) -> int:
    """
    Format all kcl files in the input directory.
    If 'recursive' is 'True', recursively search kcl files from all its
    sub directories.

    Return
    ------
    Number of formatted files
    """
    return len([kcl_fmt_file(file) for file in _get_kcl_files(file_or_dir)])


def kcl_fmt_file(filepath: pathlib.Path, is_stdout=False) -> bool:
    """
    Format single kcl file
    """
    source, is_formatted = kcl_fmt_source(filepath.read_text(encoding="utf-8"))
    if is_stdout:
        print(source, end="")
    else:
        filepath.write_text(source)
    return is_formatted


def kcl_fmt(input_file_or_path: str, is_stdout=False, recursively=False) -> List[str]:
    """
    Format kcl file or path contains kcl files

    Parameters
    ----------
    - input_file_or_path: Input filename or pathname string
    """
    try:
        changed_paths: List[str] = []
        formatting_filename = None
        path = pathlib.Path(input_file_or_path).resolve()
        if path.is_dir():
            for i, file in enumerate(_get_kcl_files(path, recursively)):
                formatting_filename = file
                if kcl_fmt_file(file, is_stdout):
                    changed_paths.append(str(file.name))
        elif path.is_file():
            formatting_filename = path
            if kcl_fmt_file(path, is_stdout):
                changed_paths.append(str(path.name))
        if not is_stdout:
            format_count = len(changed_paths)
            klog.write_out(
                "KCL format done and {} formatted:\n".format(
                    str(format_count) + " file was"
                    if format_count <= 1
                    else str(format_count) + " files were"
                )
            )
            [klog.write_out(f"{p}\n") for p in changed_paths]
        return changed_paths
    except kcl_error.KCLSyntaxException as err:
        # TODO: Support _Formatter filename context and remove this except.
        # Add filename, line, column and raise
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.InvalidSyntax_TYPE,
            file_msgs=[
                kcl_error.ErrFileMsg(
                    filename=formatting_filename, line_no=err.lineno, col_no=err.colno
                )
            ],
        )

    except Exception as err:
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.CompileError_TYPE,
            file_msgs=[
                kcl_error.ErrFileMsg(
                    filename=formatting_filename,
                )
            ],
            arg_msg=str(err),
        )
