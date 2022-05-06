# Copyright 2021 The KCL Authors. All rights reserved.

import io
from typing import List
from dataclasses import dataclass

import kclvm.kcl.ast as ast

from .token import TokenValue
from .parser import (
    Import,
    Package,
    Option,
    Field,
    OneOfField,
    OneOf,
    Map,
    Reserved,
    Range,
    EnumField,
    Enum,
    Message,
    Service,
    Rpc,
    Proto,
    ImportOption,
    Type,
    KeyType,
)


PROTO_NODE_TUPLE = (
    Import,
    Package,
    Option,
    Field,
    OneOfField,
    OneOf,
    Map,
    Reserved,
    Range,
    EnumField,
    Enum,
    Message,
    Service,
    Rpc,
    Proto,
    ImportOption,
    Type,
    KeyType,
)

_INVALID_NODE_MSG = "Invalid proto node"
WHITESPACE = " "
TAB = "\t"
NEWLINE = "\n"


# ---------------------------------------------------
# Printer config
# ---------------------------------------------------


@dataclass
class Config:
    tab_len: int = 4
    indent_len: int = 4
    use_spaces: bool = True


class BasePrinter(ast.TreeWalker):
    def __init__(self, config: Config, out: io.TextIOBase):
        self.output: str = ""
        self.config: Config = config
        self.out: io.TextIOBase = out
        self.indent: int = 0

    # Base walker functions

    def get_node_name(self, t):
        """Get the ast.AST node name"""
        return type(t).__name__

    def enter(self):
        self.indent += 1

    def leave(self):
        self.indent -= 1

    # ---------------
    # Write functions
    # ---------------

    @staticmethod
    def interleave(inter, f, seq):
        """Call f on each item in seq, calling inter() in between."""
        if not seq:
            return
        seq = iter(seq)
        f(next(seq))
        for x in seq:
            inter()
            f(x)

    def write(self, text: str = ""):
        self.out.write(text)

    def fill(self):
        self.write(
            (self.indent * self.config.indent_len * WHITESPACE)
            if self.config.use_spaces
            else (TAB * self.indent)
        )

    def print(self, *values):
        for value in values or []:
            if isinstance(value, PROTO_NODE_TUPLE):
                self.walk(value)
            elif value is True:
                self.write("true")
            elif value is False:
                self.write("false")
            else:
                self.write(str(value))


class Printer(BasePrinter):
    def __init__(self, config: Config, out: io.TextIOBase):
        super().__init__(config, out)

    def walk_Proto(self, t: Proto):
        self.print(
            TokenValue.SYNTAX,
            WHITESPACE,
            TokenValue.EQ,
            WHITESPACE,
            f'"{t.syntax}"',
            TokenValue.SEMI,
            NEWLINE,
        )
        for node in t.statements or []:
            self.walk(node)

    def walk_Import(self, t: Import):
        self.print(
            TokenValue.IMPORT,
            WHITESPACE,
            t.option.value,
            WHITESPACE,
            f'"{t.identifier}"',
            TokenValue.SEMI,
            NEWLINE,
        )

    def walk_Package(self, t: Package):
        self.print(
            TokenValue.PACKAGE,
            WHITESPACE,
        )
        self.interleave(
            lambda: self.print(TokenValue.COMMA), lambda n: self.print(n), t.identifier
        )
        self.print(
            TokenValue.SEMI,
            NEWLINE,
        )

    def walk_Option(self, t: Option):
        self.fill()
        self.print(
            TokenValue.OPTION,
            WHITESPACE,
            t.name,
            WHITESPACE,
            TokenValue.EQ,
            WHITESPACE,
            f'"{t.value}"' if isinstance(t.value, str) else t.value,
            TokenValue.SEMI,
            NEWLINE,
        )

    def walk_Enum(self, t: Enum):
        self.write(NEWLINE)
        self.fill()
        self.print(
            TokenValue.ENUM,
            WHITESPACE,
            t.name,
            WHITESPACE,
            TokenValue.LBRACE,
            NEWLINE,
        )
        self.enter()
        for node in t.body:
            self.walk(node)
        self.leave()
        self.fill()
        self.print(TokenValue.RBRACE)
        self.write(NEWLINE)

    def walk_EnumField(self, t: EnumField):
        self.fill()
        self.print(
            t.name,
            WHITESPACE,
            TokenValue.EQ,
            WHITESPACE,
            t.value,
        )
        self.write_field_options(t.options)
        self.write(TokenValue.SEMI)
        self.write(NEWLINE)

    def walk_Message(self, t: Message):
        self.write(NEWLINE)
        self.fill()
        self.print(
            TokenValue.MESSAGE,
            WHITESPACE,
            t.name,
            WHITESPACE,
            TokenValue.LBRACE,
            NEWLINE,
        )
        self.enter()
        for node in t.body:
            self.walk(node)
        self.leave()
        self.fill()
        self.print(TokenValue.RBRACE)
        self.write(NEWLINE)

    def walk_Map(self, t: Map):
        self.fill()
        self.print(
            TokenValue.MAP,
            TokenValue.LANGLE_BRACK,
            t.key_type,
            TokenValue.COMMA,
            WHITESPACE,
            t.type,
            TokenValue.RANGLE_BRACK,
            WHITESPACE,
            t.name,
            WHITESPACE,
            TokenValue.EQ,
            WHITESPACE,
            t.number,
        )
        self.write_field_options(t.options)
        self.write(TokenValue.SEMI)
        self.write(NEWLINE)

    def walk_Field(self, t: Field):
        self.fill()
        if t.repeated:
            self.print(
                TokenValue.REPEATED,
                WHITESPACE,
            )
        self.print(
            t.type, WHITESPACE, t.name, WHITESPACE, TokenValue.EQ, WHITESPACE, t.number
        )
        self.write_field_options(t.options)
        self.write(TokenValue.SEMI)
        self.write(NEWLINE)

    def walk_OneOf(self, t: OneOf):
        self.fill()
        self.print(
            TokenValue.ONEOF,
            WHITESPACE,
            t.name,
            WHITESPACE,
            TokenValue.LBRACE,
            NEWLINE,
        )
        self.enter()
        for node in t.fields:
            self.walk(node)
        self.leave()
        self.fill()
        self.print(TokenValue.RBRACE)
        self.write(NEWLINE)

    def walk_OneOfField(self, t: OneOfField):
        self.fill()
        self.print(
            t.type, WHITESPACE, t.name, WHITESPACE, TokenValue.EQ, WHITESPACE, t.number
        )
        self.write_field_options(t.options)
        self.write(TokenValue.SEMI)
        self.write(NEWLINE)

    def walk_Service(self, t: Service):
        self.write(NEWLINE)
        self.fill()
        self.print(
            TokenValue.SERVICE,
            WHITESPACE,
            t.name,
            WHITESPACE,
            TokenValue.LBRACE,
            NEWLINE,
        )
        self.enter()
        for node in t.body:
            self.walk(node)
        self.leave()
        self.fill()
        self.print(TokenValue.RBRACE)
        self.write(NEWLINE)

    def walk_Rpc(self, t: Rpc):
        self.fill()
        self.print(
            TokenValue.RPC,
            WHITESPACE,
            t.name,
            TokenValue.LPAREN,
            t.request_message_type,
            TokenValue.RPAREN,
            WHITESPACE,
            TokenValue.RETURNS,
            WHITESPACE,
            TokenValue.LPAREN,
            t.response_message_type,
            TokenValue.RPAREN,
        )
        self.write_field_options(t.options)
        self.write(TokenValue.SEMI)
        self.write(NEWLINE)

    def walk_Reserved(self, t: Reserved):
        self.fill()
        self.print(
            TokenValue().RESERVED,
            WHITESPACE,
        )
        self.interleave(
            lambda: self.write(TokenValue.COMMA + WHITESPACE),
            lambda n: self.print(f'"{n}"' if isinstance(n, str) else n),
            t.items,
        )
        self.write(TokenValue.SEMI)
        self.write(NEWLINE)

    def walk_Range(self, t: Range):
        self.print(f'"{t.from_}"' if isinstance(t.from_, str) else t.from_)
        if t.to:
            self.print(
                WHITESPACE,
                TokenValue.TO,
                WHITESPACE,
                f'"{t.to}"' if isinstance(t.to, str) else t.to,
            )

    def walk_Type(self, t: Type):
        self.print(t.value)

    def walk_KeyType(self, t: KeyType):
        self.print(t.value)

    def write_field_options(self, options: List[Option]):
        def write_option(option: Option):
            self.print(
                TokenValue.LPAREN,
                option.name,
                TokenValue.RPAREN,
                WHITESPACE,
                TokenValue.EQ,
                WHITESPACE,
                f'"{option.value}"' if isinstance(option.value, str) else option.value,
            )

        if not options:
            return
        self.write(WHITESPACE)
        self.write(TokenValue.LBRACK)
        self.interleave(
            lambda: self.write(TokenValue.COMMA + WHITESPACE), write_option, options
        )
        self.write(TokenValue.RBRACK)


def print_node_to_string(
    node,
    config: Config = Config(),
) -> str:
    """Print a proto node to string io with `config`"""
    out = io.StringIO()
    Printer(config, out).walk(node)
    return out.getvalue()
