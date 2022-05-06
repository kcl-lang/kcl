from typing import Any
from abc import abstractmethod

import kclvm.kcl.ast as ast


class BaseChecker(ast.TreeWalker):
    # checker name (you may reuse an existing one)
    name: str = ""
    # options level (0 will be displaying in --help, 1 in --long-help)
    level = 1
    # ordered list of options to control the checker behaviour
    options: Any = {}
    # messages constant to display
    MSGS: Any = {}
    # messages issued by this checker
    msgs: Any = []
    # mark this checker as enabled or not.
    enabled: bool = True

    def __init__(self, linter=None) -> None:
        """
        checker instances should have the linter as argument

        :param linter: is an object implementing KCLLinter.
        """
        if self.name is not None:
            self.name = self.name.lower()
        self.linter = linter
        self.options = linter.config if linter else None

    def __eq__(self, other) -> bool:
        return self.name == other.name and self.linter == other.linter

    def generic_walk(self, t: ast.AST) -> None:
        """Called if no explicit walker function exists for a node."""
        for _, value in ast.iter_fields(t):
            if isinstance(value, list):
                for v in value:
                    # IfStmt.elif_body: List[List[Stmt]]
                    if isinstance(v, list):
                        for v1 in v:
                            self.walk(v1)
                    if isinstance(v, ast.AST):
                        self.walk(v)
            elif isinstance(value, ast.AST):
                self.walk(value)

    def get_node_name(self, t: ast.AST) -> str:
        """Get the ast.AST node name"""
        assert isinstance(t, ast.AST)
        return t.type

    @abstractmethod
    def check(self, prog: ast.Program, code: str):
        """Should be overridden by subclass"""
        raise NotImplementedError()
