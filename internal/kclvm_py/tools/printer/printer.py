# Copyright 2021 The KCL Authors. All rights reserved.

import io
import re
import sys
from enum import IntEnum
from dataclasses import dataclass
from typing import List, Dict, Union, Tuple, Optional

import kclvm.kcl.ast as ast
import kclvm.compiler.astutil.fix as fix

# ---------------------------------------------------
# Constants
# ---------------------------------------------------


class Indentation(IntEnum):
    Indent = 0
    Dedent = 1
    Newline = 2
    IndentWithNewline = 3
    DedentWithNewline = 4
    Fill = 5


_INVALID_AST_MSG = "Invalid AST Node"
_TEMP_ROOT = "<root>"

WHITESPACE = " "
TAB = "\t"
NEWLINE = "\n"
COMMA_WHITESPACE = ast.TokenValue.COMMA + WHITESPACE
IDENTIFIER_REGEX = r"^\$?[a-zA-Z_]\w*$"
QUANT_OP_TOKEN_VAL_MAPPING = {
    ast.QuantOperation.ALL: ast.TokenValue.ALL,
    ast.QuantOperation.ANY: ast.TokenValue.ANY,
    ast.QuantOperation.MAP: ast.TokenValue.MAP,
    ast.QuantOperation.FILTER: ast.TokenValue.FILTER,
}

# ---------------------------------------------------
# Printer config
# ---------------------------------------------------


@dataclass
class Config:
    tab_len: int = 4
    indent_len: int = 4
    use_spaces: bool = True
    is_fix: bool = False


# ---------------------------------------------------
# Printer
# ---------------------------------------------------


class BasePrinter(ast.TreeWalker):
    def __init__(self, config: Config, out: io.TextIOBase):
        self.output: str = ""
        self.indent = 0
        self.cfg: Config = config
        self.out: io.TextIOBase = out
        # Print comments
        self.last_ast_line: int = 0
        self.comments: List[ast.Comment] = []
        self.import_spec: Dict[str, str] = {}

    # Base walker functions

    def get_node_name(self, t: ast.AST):
        """Get the ast.AST node name"""
        assert isinstance(t, ast.AST)
        return t.type

    def generic_walk(self, t: ast.AST):
        """Called if no explicit walker function exists for a node."""
        if not isinstance(t, ast.AST):
            raise Exception(_INVALID_AST_MSG, t)
        else:
            self.walk(t)

    def write_ast_comments(self, t: ast.AST):
        if not t or not isinstance(t, ast.AST):
            return
        if t.line and t.line > self.last_ast_line:
            self.last_ast_line = t.line
            if self.comments:
                index = -1
                for i, comment in enumerate(self.comments):
                    if comment.line <= t.line:
                        index = i
                        self.write(comment.text + NEWLINE)
                        self.fill()
                    else:
                        break
                if index >= 0:
                    del self.comments[: index + 1]

    # -----------------
    # Expr and Stmt walker functions
    # -----------------

    def expr(self, expr: ast.Expr):
        if not expr:
            return
        self.write_ast_comments(expr)
        self.generic_walk(expr)

    def stmt(self, stmt: ast.Stmt):
        if not stmt:
            return
        self.fill()
        self.write_ast_comments(stmt)
        self.generic_walk(stmt)

    def stmts(self, stmts: List[ast.Stmt]):
        for stmt in stmts or []:
            self.stmt(stmt)

    def exprs(self, exprs: List[ast.Expr]):
        for expr in exprs or []:
            self.expr(expr)

    # --------------------------
    # Indent and scope functions
    # --------------------------

    def enter(self):
        self.indent += 1

    def leave(self):
        self.indent -= 1

    # --------------------------
    # Write functions
    # --------------------------

    @staticmethod
    def interleave(inter, f, seq):
        """Call f on each item in seq, calling inter() in between."""
        if not seq:
            return
        seq = iter(seq)
        try:
            f(next(seq))
        except StopIteration:
            pass
        else:
            for x in seq:
                inter()
                f(x)

    def write(self, text: str = ""):
        self.out.write(text)

    def writeln(self, text: str = ""):
        self.write(text + NEWLINE)
        self.fill()

    def write_with_spaces(self, text: str = ""):
        self.write(" " + text + " ")

    def fill(self, text: str = ""):
        """Indent a piece of text, according to the current indentation level"""
        if self.cfg.use_spaces:
            self.write(WHITESPACE * self.indent * self.cfg.indent_len + text)
        else:
            self.write(TAB * self.indent + text)

    def print(self, *values):
        for value in values or []:
            if isinstance(value, ast.Expr):
                self.expr(value)
            elif isinstance(value, ast.Stmt):
                self.stmt(value)
            elif isinstance(value, ast.AST):
                self.generic_walk(value)
            elif value == Indentation.Indent:
                self.enter()
            elif value == Indentation.Dedent:
                self.leave()
            elif value == Indentation.Newline:
                self.writeln()
            elif value == Indentation.IndentWithNewline:
                self.enter()
                self.writeln()
            elif value == Indentation.DedentWithNewline:
                self.leave()
                self.writeln()
            elif value == Indentation.Fill:
                self.fill()
            elif isinstance(value, str):
                self.write(value)
            elif isinstance(value, (int, float, bool)):
                self.write(str(value))

    def print_ast(self, t: ast.AST):
        if not t:
            return
        if isinstance(t, ast.Module) and self.cfg.is_fix:
            fix.fix_and_get_module_import_list(
                _TEMP_ROOT, t, is_fix=True, reversed=True
            )
        self.walk(t)
        for comment in self.comments or []:
            self.write(comment.text + NEWLINE)
            self.fill()


class Printer(BasePrinter):
    def __init__(self, config: Config, out: io.TextIOBase):
        super().__init__(config, out)

    def walk_Module(self, t: ast.Module):
        """ast.AST: Module

        Parameters
        ----------
        - body: List[Stmt]
        """
        assert isinstance(t, ast.Module)
        self.comments = t.comments
        self.stmts(t.body)

    def walk_ExprStmt(self, t: ast.ExprStmt):
        """ast.AST: ExprStmt

        Parameters
        ----------
        - exprs: List[Expr]
        """
        assert isinstance(t, ast.ExprStmt)
        self.interleave(lambda: self.write(COMMA_WHITESPACE), self.expr, t.exprs)
        self.writeln()

    def walk_AssertStmt(self, t: ast.AssertStmt):
        """ast.AST: AssertStmt

        Parameters
        ----------
        - test: Expr
        - if_cond: Expr
        - msg: Expr
        """
        assert isinstance(t, ast.AssertStmt) and t.test
        self.print(
            ast.TokenValue.ASSERT,
            WHITESPACE,
            t.test,
        )
        if t.if_cond:
            self.print(WHITESPACE, ast.TokenValue.IF, WHITESPACE, t.if_cond)
        if t.msg:
            self.print(
                COMMA_WHITESPACE,
                t.msg,
            )
        self.print(NEWLINE)

    def walk_IfStmt(self, t: ast.IfStmt):
        """ast.AST: IfStmt

        Parameters
        ----------
        - cond: Expr
        - body: List[Stmt]
        - elif_cond: List[Expr]
        - elif_body: List[List[Stmt]]
        - else_body: List[Stmt]
        """
        assert isinstance(t, ast.IfStmt)
        assert t.cond
        assert t.body
        self.print(
            ast.TokenValue.IF,
            WHITESPACE,
            t.cond,
            ast.TokenValue.COLON,
            NEWLINE,
            Indentation.Indent,
        )
        self.stmts(t.body)
        self.print(Indentation.Dedent)
        if t.elif_cond:
            for cond, body in zip(t.elif_cond, t.elif_body):
                # Nested if statements need to be considered,
                # so `elif` needs to be preceded by the current indentation.
                self.print(
                    Indentation.Fill,
                    ast.TokenValue.ELIF,
                    WHITESPACE,
                    cond,
                    ast.TokenValue.COLON,
                    NEWLINE,
                    Indentation.Indent,
                )
                self.stmts(body)
                self.print(Indentation.Dedent)
        if t.else_body:
            # Nested if statements need to be considered,
            # so `else` needs to be preceded by the current indentation.
            self.print(
                Indentation.Fill,
                ast.TokenValue.ELSE,
                ast.TokenValue.COLON,
                NEWLINE,
                Indentation.Indent,
            )
            self.stmts(t.else_body)
            self.print(Indentation.Dedent)

    def walk_ImportStmt(self, t: ast.ImportStmt):
        """ast.AST: ImportStmt

        Parameters
        ---------
        - path: str
        - name: str
        - asname: str
        """
        assert isinstance(t, ast.ImportStmt)
        assert t.pkg_name
        self.print(
            ast.TokenValue.IMPORT,
            WHITESPACE,
            t.path,
        )
        if t.asname:
            self.print(
                WHITESPACE,
                ast.TokenValue.AS,
                WHITESPACE,
                t.asname,
            )
        self.import_spec[t.path] = t.pkg_name
        self.writeln()

    def walk_SchemaStmt(self, t: ast.SchemaStmt):
        """ast.AST: SchemaStmt

        Parameters
        ----------
        - doc: str
        - name: str
        - parent_name: Identifier
        - is_mixin: bool
        - is_protocol: bool
        - args: Arguments
        - settings: dict
        - mixins: List[str]
        - body: List[Union[SchemaAttr, Stmt]]
        - decorators: List[Decorator]
        - checks: List[CheckExpr]
        - for_host_name: Optional[Identifier] = None
        - index_signature: Optional[SchemaIndexSignature] = None
        """
        assert isinstance(t, ast.SchemaStmt)
        self.exprs(t.decorators)
        tok = ast.TokenValue.SCHEMA
        if t.is_mixin:
            tok = ast.TokenValue.MIXIN
        elif t.is_protocol:
            tok = ast.TokenValue.PROTOCOL
        self.print(
            tok,
            WHITESPACE,
        )
        self.print(t.name)
        if t.args:
            self.print(
                ast.TokenValue.LEFT_BRACKETS,
                t.args,
                ast.TokenValue.RIGHT_BRACKETS,
            )
        if t.parent_name:
            self.print(
                ast.TokenValue.LEFT_PARENTHESES,
                t.parent_name,
                ast.TokenValue.RIGHT_PARENTHESES,
            )
        if t.for_host_name:
            self.print(
                WHITESPACE,
                ast.TokenValue.FOR,
                WHITESPACE,
                t.for_host_name,
            )
        self.print(
            ast.TokenValue.COLON,
            NEWLINE,
            Indentation.Indent,  # Schema Stmt indent
        )
        if t.doc:
            self.print(
                Indentation.Fill,
                '"""',
                t.doc.replace('"', '\\"'),
                '"""',
                NEWLINE,
            )
        if t.mixins:
            self.print(
                Indentation.Fill,
                ast.TokenValue.MIXIN,
                WHITESPACE,
                ast.TokenValue.LEFT_BRACKETS,
                Indentation.IndentWithNewline,
            )
            self.interleave(
                lambda: self.print(ast.TokenValue.COMMA, Indentation.Newline),
                self.print,
                t.mixins,
            )
            self.print(
                Indentation.Dedent,
                Indentation.Newline,
                ast.TokenValue.RIGHT_BRACKETS,
                NEWLINE,
            )
        if t.index_signature:
            self.print(t.index_signature)
            self.print(NEWLINE)
        self.stmts(t.body)
        self.write(NEWLINE)
        if t.checks:
            self.print(
                Indentation.Fill,
                ast.TokenValue.CHECK,
                ast.TokenValue.COLON,
                Indentation.IndentWithNewline,  # Schema check indent
            )
            self.interleave(
                lambda: self.print(Indentation.Newline),
                self.print,
                t.checks,
            )
            self.write(NEWLINE)
            # Schema check dedent
            self.print(Indentation.Dedent)
            self.write(NEWLINE)
        # Schema Stmt dedent
        self.print(Indentation.Dedent)

    def walk_SchemaIndexSignature(self, t: ast.SchemaIndexSignature):
        """ast.AST: SchemaIndexSignature

        Parameters
        ----------
        - key_name: Optional[str]
        - key_type: Optional[str]
        - value_type: Optional[str]
        - value: Optional[Expr]
        - any_other: bool
        - name_node: Optional[Name]
        - value_type_node: Optional[Type]
        """
        self.write(ast.TokenValue.LEFT_BRACKETS)
        if t.any_other:
            self.write("...")
        if t.key_name:
            self.print(
                t.key_name,
                ast.TokenValue.COLON,
                WHITESPACE,
            )
        self.print(
            t.key_type,
            ast.TokenValue.RIGHT_BRACKETS,
            ast.TokenValue.COLON,
            WHITESPACE,
            t.value_type,
        )
        if t.value:
            self.print(WHITESPACE, ast.TokenValue.ASSIGN, WHITESPACE, t.value)

    def walk_RuleStmt(self, t: ast.RuleStmt):
        """ast.AST: RuleStmt

        Parameters
        ----------
        self.doc: str = ""
        self.name: str = ""
        self.parent_rules: List[Identifier] = []
        self.decorators: List[Decorator] = []
        self.checks: List[CheckExpr] = []
        self.name_node: Optional[Name] = None
        self.args: Optional[Arguments] = None
        self.for_host_name: Optional[Identifier] = None
        """
        assert isinstance(t, ast.RuleStmt)
        self.exprs(t.decorators)
        self.print(
            ast.TokenValue.RULE,
            WHITESPACE,
        )
        self.print(t.name)
        if t.args:
            self.print(
                ast.TokenValue.LEFT_BRACKETS,
                t.args,
                ast.TokenValue.RIGHT_BRACKETS,
            )
        if t.parent_rules:
            self.print(ast.TokenValue.LEFT_PARENTHESES)
            self.interleave(
                lambda: self.print(ast.TokenValue.COMMA, WHITESPACE),
                self.print,
                t.parent_rules,
            )
            self.print(ast.TokenValue.RIGHT_PARENTHESES)
        if t.for_host_name:
            self.print(
                WHITESPACE,
                ast.TokenValue.FOR,
                WHITESPACE,
                t.for_host_name,
            )
        self.print(
            ast.TokenValue.COLON,
            Indentation.IndentWithNewline,  # Schema Stmt indent
        )
        if t.doc:
            self.print(
                '"""',
                t.doc.replace('"', '\\"'),
                '"""',
                Indentation.Newline,
            )
        if t.checks:
            self.interleave(
                lambda: self.print(Indentation.Newline),
                self.print,
                t.checks,
            )
        self.write(NEWLINE)
        # Schema Stmt dedent
        self.print(Indentation.Dedent)

    def walk_Decorator(self, t: ast.Decorator):
        """ast.AST: Decorator

        Parameters
        ----------
        - name: Optional[Identifier]
        - args: Optional[CallExpr]
        """
        assert isinstance(t, ast.Decorator)
        self.print(
            ast.TokenValue.AT,
            t.name,
        )
        if t.args:
            self.print(
                ast.TokenValue.LEFT_PARENTHESES,
                t.args,
                ast.TokenValue.RIGHT_PARENTHESES,
            )
        self.writeln()

    def walk_Arguments(self, t: ast.Arguments):
        """ast.AST: Arguments

        Parameters
        ----------
        - args: List[Identifier] = []
        - type_annotation_list: List[str] = []
        - defaults: List[Expr] = []
        """
        assert isinstance(t, ast.Arguments)

        def write_argument(
            para: Tuple[ast.Identifier, Optional[str], Optional[ast.Expr]]
        ):
            arg, type_str, default = para
            self.print(
                arg,
                (": " + type_str) if type_str else "",
            )
            if default:
                self.print(WHITESPACE, ast.TokenValue.ASSIGN, WHITESPACE, default)

        self.interleave(
            lambda: self.write(COMMA_WHITESPACE),
            write_argument,
            zip(t.args, t.type_annotation_list, t.defaults),
        )

    def walk_SchemaAttr(self, t: ast.SchemaAttr):
        """ast.AST: SchemaAttr

        Parameters
        ----------
        - doc: str
        - name: str
        - type_str: str
        - is_optional: bool
        - value: Expr
        - decorators: List[Decorator]
        - op: Union[AugOp, Assign]
        """
        assert isinstance(t, ast.SchemaAttr)
        self.exprs(t.decorators)
        self.print(
            t.name,
            ast.TokenValue.QUESTION if t.is_optional else "",
        )
        self.print(ast.TokenValue.COLON, WHITESPACE, t.type_str)
        if t.op:
            self.print(
                WHITESPACE,
                ast.OPERATOR_VALUE_MAP[t.op],
                WHITESPACE,
                t.value,
            )
        self.write(NEWLINE)

    def walk_IfExpr(self, t: ast.IfExpr):
        """ast.AST: IfExpr

        Parameters
        ----------
        - cond: Expr
        - body: Expr
        - orelse: Expr
        """
        assert isinstance(t, ast.IfExpr)
        self.print(
            t.body,
            WHITESPACE,
            ast.TokenValue.IF,
            WHITESPACE,
            t.cond,
            WHITESPACE,
            ast.TokenValue.ELSE,
            WHITESPACE,
            t.orelse,
        )

    def walk_UnaryExpr(self, t: ast.UnaryExpr):
        """ast.AST: UnaryExpr(Expr)

        Parameters
        ----------
        - op: UnaryOp
        - operand: Expr
        """
        assert isinstance(t, ast.UnaryExpr)
        self.print(
            ast.OPERATOR_VALUE_MAP[t.op],
            WHITESPACE if t.op == ast.UnaryOp.Not else "",
            t.operand,
        )

    def walk_BinaryExpr(self, t: ast.BinaryExpr):
        """ast.AST: BinaryExpr

        Parameters
        ----------
        - left: Expr
        - right: Expr
        - op: BinaryOperator
        """
        assert isinstance(t, ast.BinaryExpr) and t.left and t.right and t.op
        self.print(
            t.left,
            WHITESPACE,
            ast.OPERATOR_VALUE_MAP[t.op],
            WHITESPACE,
            t.right,
        )

    def walk_SelectorExpr(self, t: ast.SelectorExpr):
        """ast.AST: SelectorExpr

        Parameters
        ----------
        - value: Expr
        - attr: Identifier
        - ctx: ExprContext
        - has_question: bool
        """
        assert isinstance(t, ast.SelectorExpr)
        self.print(
            t.value,
            ast.TokenValue.QUESTION if t.has_question else "",
            ast.TokenValue.DOT,
            t.attr,
        )

    def walk_CallExpr(self, t: ast.CallExpr):
        """ast.AST: CallExpr

        Parameters
        ----------
        - func: Expr
        - args: List[Expr]
        - keywords: List[Keyword]
        """
        assert isinstance(t, ast.CallExpr)
        if t.func:
            self.print(
                t.func,
                ast.TokenValue.LEFT_PARENTHESES,
            )
        self.write_args_and_kwargs(t.args, t.keywords)
        if t.func:
            self.print(
                ast.TokenValue.RIGHT_PARENTHESES,
            )

    def walk_ParenExpr(self, t: ast.ParenExpr):
        """ast.AST: ParenExpr

        Parameters
        ----------
        - expr: Expr
        """
        assert isinstance(t, ast.ParenExpr)
        self.print(
            ast.TokenValue.LEFT_PARENTHESES,
            t.expr,
            ast.TokenValue.RIGHT_PARENTHESES,
        )

    def walk_ListExpr(self, t: ast.ListExpr):
        """ast.AST: ListExpr

        Parameters
        ----------
        - elts: List[Expr]
        """
        assert isinstance(t, ast.ListExpr)
        in_one_line = len(set(map(lambda e: e.line, t.elts)).union([t.line])) == 1
        self.write(ast.TokenValue.LEFT_BRACKETS)
        if t.elts:
            self.print(Indentation.IndentWithNewline if not in_one_line else "")
            splits = COMMA_WHITESPACE if in_one_line else Indentation.Newline
            self.interleave(
                lambda: self.print(splits),
                self.expr,
                t.elts,
            )
            self.print(Indentation.DedentWithNewline if not in_one_line else "")
        self.write(ast.TokenValue.RIGHT_BRACKETS)

    def walk_StarredExpr(self, t: ast.StarredExpr):
        assert isinstance(t, ast.StarredExpr) and t.value
        self.print(
            ast.TokenValue.MULTIPLY,
            t.value,
        )

    def walk_ListComp(self, t: ast.ListComp):
        """ast.AST: ListComp

        Parameters
        ----------
        - elt: Expr
        - generators: List[CompClause]
            - targets: List[Expr]
            - iter: Expr
            - ifs: List[Expr]
        """
        assert isinstance(t, ast.ListComp)
        self.write(ast.TokenValue.LEFT_BRACKETS)
        self.expr(t.elt)
        for gen in t.generators:
            self.expr(gen)
        self.write(ast.TokenValue.RIGHT_BRACKETS)

    def walk_DictComp(self, t: ast.DictComp):
        """ast.AST: DictComp

        Parameters
        ----------
        - key: Expr
        - value: Expr
        - generators: List[CompClause]
        """
        assert isinstance(t, ast.DictComp)
        self.write(ast.TokenValue.LEFT_BRACE)
        self.print(
            t.key,
            ast.TokenValue.COLON,
            WHITESPACE,
            t.value,
        )
        for gen in t.generators:
            self.expr(gen)
        self.write(ast.TokenValue.RIGHT_BRACE)

    def walk_CompClause(self, t: ast.CompClause):
        """ast.AST: CompClause

        Parameters
        ----------
        - targets: List[Expr]
        - iter: Expr
        - ifs: List[Expr]
        """
        assert isinstance(t, ast.CompClause)
        self.write_with_spaces(ast.TokenValue.FOR)
        self.interleave(lambda: self.write(COMMA_WHITESPACE), self.expr, t.targets)
        self.write_with_spaces(ast.TokenValue.IN)
        self.expr(t.iter)
        for if_clause in t.ifs:
            self.write_with_spaces(ast.TokenValue.IF)
            self.expr(if_clause)

    def walk_QuantExpr(self, t: ast.QuantExpr):
        """ast.AST: QuantExpr

        Parameters
        ----------
        - target: Optional[Expr] = None
        - variables: List[Identifier] = []
        - op: Optional[int] = None
        - test: Optional[Expr] = None
        - if_cond: Optional[Expr] = None
        - ctx: ExprContext = ctx
        """
        in_one_line = t.test.line == t.line
        self.print(
            QUANT_OP_TOKEN_VAL_MAPPING[t.op],
            WHITESPACE,
        )
        self.interleave(lambda: self.write(COMMA_WHITESPACE), self.expr, t.variables)
        self.write_with_spaces(ast.TokenValue.IN)
        self.expr(t.target)
        self.write(WHITESPACE)
        self.write(ast.TokenValue.LEFT_BRACE)
        self.print(Indentation.IndentWithNewline if not in_one_line else "")
        self.expr(t.test)
        if t.if_cond:
            self.print(WHITESPACE, ast.TokenValue.IF, WHITESPACE, t.if_cond)
        self.print(Indentation.DedentWithNewline if not in_one_line else "")
        self.write(ast.TokenValue.RIGHT_BRACE)

    def walk_Subscript(self, t: ast.Subscript):
        """ast.AST: Subscript

        Parameters
        ----------
        - value: Optional[Expr] = None
        - index: Optional[Expr] = None
        - lower: Optional[Expr] = None
        - upper: Optional[Expr] = None
        - step: Optional[Expr] = None
        - ctx: ExprContext = ExprContext.LOAD
        - has_question: bool = False
        """
        assert isinstance(t, ast.Subscript)
        self.print(
            t.value,
            ast.TokenValue.QUESTION if t.has_question else "",
            ast.TokenValue.LEFT_BRACKETS,
        )
        if t.index:
            self.expr(t.index)
        else:
            self.print(
                t.lower,
                ast.TokenValue.COLON,
                t.upper,
                ast.TokenValue.COLON,
                t.step,
            )
        self.print(ast.TokenValue.RIGHT_BRACKETS)

    def walk_SchemaExpr(self, t: ast.SchemaExpr):
        """ast.AST: SchemaExpr

        Parameters
        ----------
        - name: Identifier
        - config: ConfigExpr
        - args: List[Expr] = []
        - kwargs: List[Keyword] = []
        """
        assert isinstance(t, ast.SchemaExpr)
        self.print(t.name)

        if t.args or t.kwargs:
            self.write(ast.TokenValue.LEFT_PARENTHESES)
            self.write_args_and_kwargs(t.args, t.kwargs)
            self.write(ast.TokenValue.RIGHT_PARENTHESES)

        self.print(WHITESPACE, t.config)

    def walk_ConfigExpr(self, t: ast.ConfigExpr):
        """ast.AST: ConfigExpr

        Parameters
        ----------
        - items: List[ConfigEntry] = []
            - key: Expr = key
            - value: Expr = value
            - operation: int = ConfigEntryOperation.UNION
            - insert_index: Union[int, str] = -1
        """

        def write_config_key(key: ast.AST) -> int:
            """Write config key and return need right brace"""
            if isinstance(key, ast.Identifier):
                self.write_ast_comments(key)
                names = key.names
                # Judge contains string identifier, e.g., "x-y-z"
                need_right_brace = not all(
                    [bool(re.match(IDENTIFIER_REGEX, n)) for n in names]
                )
                if need_right_brace:
                    # a: {b: {c op value}}
                    self.print(
                        ": {".join(
                            ['"{}"'.format(n.replace('"', '\\"')) for n in names]
                        )
                    )
                    return len(names) - 1
                else:
                    # a.b.c op value
                    self.expr(key)
                    return 0
            else:
                self.expr(key)
                return 0

        def write_item(item: ast.ConfigEntry):
            if item.key is None:
                # for dictionary unpacking operator in dicts {**{'y': 2}}
                # see PEP 448 for details
                if not isinstance(item.value, ast.ConfigIfEntryExpr):
                    self.print(ast.TokenValue.DOUBLE_STAR)
                self.print(item.value)
            else:
                tok = ast.TokenValue.COLON
                if item.operation == ast.ConfigEntryOperation.INSERT:
                    tok = ast.TokenValue.COMP_PLUS
                elif item.operation == ast.ConfigEntryOperation.OVERRIDE:
                    tok = ast.TokenValue.ASSIGN
                print_right_brace_count = write_config_key(item.key)
                if item.insert_index is not None and item.insert_index >= 0:
                    self.print(
                        ast.TokenValue.LEFT_BRACKETS,
                        item.insert_index,
                        ast.TokenValue.RIGHT_BRACKETS,
                    )
                if tok != ast.TokenValue.COLON:
                    self.print(WHITESPACE)
                self.print(
                    tok,
                    WHITESPACE,
                    item.value,
                )
                self.print(ast.TokenValue.RIGHT_BRACE * (print_right_brace_count or 0))

        in_one_line = len(set(map(lambda e: e.line, t.items)).union([t.line])) == 1
        self.write(ast.TokenValue.LEFT_BRACE)
        if t.items:
            self.print(Indentation.IndentWithNewline if not in_one_line else "")
            self.interleave(
                lambda: self.print(COMMA_WHITESPACE) if in_one_line else self.writeln(),
                write_item,
                t.items,
            )
            self.print(Indentation.DedentWithNewline if not in_one_line else "")
        self.write(ast.TokenValue.RIGHT_BRACE)

    def walk_CheckExpr(self, t: ast.CheckExpr):
        """ast.AST: CheckExpr

        Parameters
        ----------
        - test: Expr
        - if_cond: Expr
        - msg: Expr
        """
        assert isinstance(t, ast.CheckExpr) and t.test
        self.expr(t.test)
        if t.if_cond:
            self.print(WHITESPACE, ast.TokenValue.IF, WHITESPACE, t.if_cond)
        if t.msg:
            self.print(
                COMMA_WHITESPACE,
                t.msg,
            )

    def walk_LambdaExpr(self, t: ast.LambdaExpr):
        """ast.AST: LambdaExpr

        Parameters
        ----------
        - args: Optional[Arguments]
        - return_type_str: Optional[str]
        - return_type_node: Optional[Type]
        - body: List[Stmt]
        """
        self.print(ast.TokenValue.LAMBDA)
        if t.args:
            self.print(
                WHITESPACE,
                t.args,
            )
        if t.return_type_str:
            self.print(
                WHITESPACE,
                ast.TokenValue.RIGHT_ARROW,
                WHITESPACE,
                t.return_type_str,
            )
        self.print(
            WHITESPACE,
            ast.TokenValue.LEFT_BRACE,
            NEWLINE,
            Indentation.Indent,
        )
        self.stmts(t.body)
        self.print(
            Indentation.Dedent,
            NEWLINE,
            ast.TokenValue.RIGHT_BRACE,
        )

    def walk_Compare(self, t: ast.Compare):
        """ast.AST: Compare

        Parameters
        ----------
        - left: Optional[Expr] = None
        - ops: List[CmpOp] = []
        - comparators: List[Expr] = []
        """
        assert isinstance(t, ast.Compare)
        self.expr(t.left)
        for op, expr in zip(t.ops, t.comparators):
            self.print(
                WHITESPACE,
                ast.OPERATOR_VALUE_MAP[op],
                WHITESPACE,
                expr,
            )

    def walk_Identifier(self, t: ast.Identifier):
        """ast.AST: Identifier

        Parameters
        ----------
        - names: List[str]
        """
        assert isinstance(t, ast.Identifier) and t.ctx
        # Convert pkgpath qualified name to a normal identifier
        if t.names[0].startswith("@"):
            pkgpath = t.names[0][1:]
            t.names[0] = self.import_spec.get(pkgpath) or pkgpath
        self.write(t.get_name())

    def walk_NumberLit(self, t: ast.AST):
        """ast.AST: NumberLit

        Parameters
        ----------
        - value: int
        """
        assert isinstance(t, ast.NumberLit)
        self.write(str(t.value))

    def walk_StringLit(self, t: ast.StringLit):
        """ast.AST: StringLit

        Parameters
        ----------
        - value: str
        - raw_value: str
        - is_long_string = False
        """
        assert isinstance(t, ast.StringLit)
        self.write(
            t.raw_value
            or (
                '"""{}"""'.format(t.value.replace('"', '\\"'))
                if t.is_long_string
                else '"{}"'.format(t.value.replace('"', '\\"'))
            )
        )

    def walk_NameConstantLit(self, t: ast.NameConstantLit):
        """ast.AST: NameConstantLit

        Parameters
        ----------
        - value
        """
        assert isinstance(t, ast.NameConstantLit)
        # None, Undefined, True, False
        self.write(str(t.value))

    def walk_JoinedString(self, t: ast.JoinedString):
        """ast.AST: JoinedString

        Parameters
        ----------
        - values: List[Union[Expr, StringLit]]

        TOS
        ---
        - format_spec
        - formatted expr list

        Operand
        -------
        Formatted expr list count
        """
        assert isinstance(t, ast.JoinedString)
        assert t.values
        quote_str = '"""' if t.is_long_string else '"'
        self.print(quote_str)
        for value in t.values:
            if isinstance(value, ast.FormattedValue):
                self.print(
                    "$",
                    ast.TokenValue.LEFT_BRACE,
                    value.value,
                )
                if value.format_spec:
                    self.print(
                        ast.TokenValue.COLON,
                        WHITESPACE,
                        value.format_spec,
                    )
                self.print(
                    ast.TokenValue.RIGHT_BRACE,
                )
            elif isinstance(value, ast.StringLit):
                self.write(
                    value.raw_value or "{}".format(value.value.replace('"', '\\"'))
                )
            elif isinstance(value, ast.Expr):
                self.expr(value)
            else:
                raise Exception("Invalid AST JoinedString children")
        self.print(quote_str)

    def walk_TypeAliasStmt(self, t: ast.TypeAliasStmt):
        """ast.AST: TypeAliasStmt

        Parameters
        ----------
        - type_name: Identifier
        - type_value: Type
        """
        self.print(
            ast.TokenValue.TYPE,
            WHITESPACE,
            t.type_name,
            WHITESPACE,
            ast.TokenValue.ASSIGN,
            WHITESPACE,
            t.type_value.plain_type_str,
            NEWLINE,
        )

    def walk_UnificationStmt(self, t: ast.UnificationStmt):
        """ast.AST: UnificationStmt

        Parameters
        ----------
        - target: Identifier
        - value: SchemaExpr
        """
        self.print(t.target, ast.TokenValue.COLON, WHITESPACE, t.value, NEWLINE)

    def walk_AssignStmt(self, t: ast.AssignStmt):
        """ast.AST: AssignStmt

        Parameters
        ----------
        - targets: List[Identifier]
        - value: Expr
        """
        assert isinstance(t, ast.AssignStmt) and t.targets and t.value
        for i, target in enumerate(t.targets):
            self.print(target)
            if i == 0 and t.type_annotation:
                self.print(ast.TokenValue.COLON, WHITESPACE, t.type_annotation)
            self.print(WHITESPACE, ast.TokenValue.ASSIGN, WHITESPACE)
        self.print(t.value, NEWLINE)
        if isinstance(t.value, ast.SchemaExpr):
            self.print(NEWLINE)

    def walk_AugAssignStmt(self, t: ast.AugAssignStmt):
        """ast.AST: AugAssignStmt

        Parameters
        ----------
        - target: Identifier
        - value: Expr
        - op: AugOp
        """
        assert isinstance(t, ast.AugAssignStmt) and t.target and t.value and t.op
        self.print(
            t.target,
            WHITESPACE,
            ast.OPERATOR_VALUE_MAP[t.op],
            WHITESPACE,
            t.value,
            NEWLINE,
        )

    def write_args_and_kwargs(self, args: List[ast.Expr], keywords: List[ast.Keyword]):
        def print_arg_assign_value(keyword: ast.Keyword):
            self.print(
                keyword.arg,
                ast.TokenValue.ASSIGN,
                keyword.value,
            )

        self.interleave(lambda: self.write(COMMA_WHITESPACE), self.expr, args)
        if args and keywords:
            self.print(COMMA_WHITESPACE)
        self.interleave(
            lambda: self.write(COMMA_WHITESPACE), print_arg_assign_value, keywords
        )

    def walk_ListIfItemExpr(self, t: ast.ListIfItemExpr):
        assert isinstance(t, ast.ListIfItemExpr)
        self.print(
            ast.TokenValue.IF,
            WHITESPACE,
            t.if_cond,
            ast.TokenValue.COLON,
            Indentation.IndentWithNewline,
        )
        self.interleave(lambda: self.print(NEWLINE), self.print, t.exprs)
        self.print(Indentation.DedentWithNewline)
        if t.orelse:
            if isinstance(t.orelse, ast.ListIfItemExpr):
                self.print("el")
                self.expr(t.orelse)
            elif isinstance(t.orelse, ast.ListExpr):
                self.print(
                    ast.TokenValue.ELSE,
                    ast.TokenValue.COLON,
                    Indentation.IndentWithNewline,
                )
                self.interleave(lambda: self.print(NEWLINE), self.print, t.orelse.elts)
                self.print(Indentation.Dedent)

    def walk_ConfigIfEntryExpr(self, t: ast.ConfigIfEntryExpr):
        assert isinstance(t, ast.ConfigIfEntryExpr)
        self.print(
            ast.TokenValue.IF,
            WHITESPACE,
            t.if_cond,
            ast.TokenValue.COLON,
            Indentation.IndentWithNewline,
        )

        def write_item(item):
            key, value, operation = item
            tok = ast.TokenValue.COLON
            if operation == ast.ConfigEntryOperation.INSERT:
                tok = ast.TokenValue.COMP_PLUS
            elif operation == ast.ConfigEntryOperation.OVERRIDE:
                tok = ast.TokenValue.ASSIGN
            self.print(key)
            if tok != ast.TokenValue.COLON:
                self.print(WHITESPACE)
            self.print(
                tok,
                WHITESPACE,
                value,
            )

        self.interleave(
            lambda: self.writeln(),
            write_item,
            zip(t.keys, t.values, t.operations),
        )
        self.print(Indentation.DedentWithNewline)
        if t.orelse:
            if isinstance(t.orelse, ast.ConfigIfEntryExpr):
                self.print("el")
                self.expr(t.orelse)
            elif isinstance(t.orelse, ast.ConfigExpr):
                self.print(
                    ast.TokenValue.ELSE,
                    ast.TokenValue.COLON,
                    Indentation.IndentWithNewline,
                )
                self.interleave(
                    lambda: self.print(Indentation.Newline),
                    write_item,
                    zip(t.orelse.keys, t.orelse.values, t.orelse.operations),
                )
                self.print(Indentation.Dedent)


def PrintAST(
    node: ast.AST,
    out: Union[io.TextIOBase, io.StringIO] = sys.stdout,
    config: Config = Config(),
):
    """Print a KCL AST Module to `out` io with `config`"""
    Printer(config, out).print_ast(node)
