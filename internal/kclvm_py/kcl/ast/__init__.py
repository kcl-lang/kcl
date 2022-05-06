"""The `ast` module mainly defines the abstract nodes of all
KCL syntax and the corresponding supporting tools that make
working with the trees simpler.

The syntax tree can be generated through functions such as
ParseFile/LoadProgram, and the result will be a tree of
objects whose classes all inherit from `ast.AST`.

In addition to the grammar model itself, the `ast` module
also defines the priority of all KCL operators, as well as
the walker and transformer modules that help process the AST.
The former is used to better traverse the AST, and the latter
is used to modify the existing AST more quickly.

:copyright: Copyright 2020 The KCL Authors. All rights reserved.
"""

from .ast import *
from .precedence import OP_PREC_MAP, precedence
from .lark_token import TokenValue
from .walker import TreeWalker, WalkTree
from .transformer import TreeTransformer
from .fields_map import iter_fields

__all__ = [
    "ast",
    "BinOp",
    "CmpOp",
    "UnaryOp",
    "AugOp",
    "ExprContext",
    "OP_PREC_MAP",
    "precedence",
    "TokenValue",
    "TreeWalker",
    "WalkTree",
    "TreeTransformer",
    "iter_fields",
    "ASTFactory",
]
