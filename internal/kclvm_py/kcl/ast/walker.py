# Copyright 2020 The KCL Authors. All rights reserved.

import typing
from abc import abstractmethod

import kclvm.kcl.ast as ast


class TreeWalker:
    """The TreeWalk class can be used as a superclass in order
    to walk an AST or similar tree.

    This class is meant to be subclassed, with the subclass adding walker
    methods.

    Per default the walker functions for the nodes are ``'walk_'`` +
    class name of the node.  So a `expr_stmt` node visit function would
    be `walk_expr_stmt`.  This behavior can be changed by overriding
    the `walk` method.  If no walker function exists for a node
    (return value `None`) the `generic_walker` walker is used instead.
    """

    _WALK_FUNCTION_PREFIX = "walk_"

    def walk(self, node) -> None:
        """Visit a node."""
        method = self._WALK_FUNCTION_PREFIX + self.get_node_name(node)
        walker = getattr(self, method, self.generic_walk)
        return walker(node)

    @abstractmethod
    def generic_walk(self, node):
        """Called if no explicit walker function exists for a node."""
        pass

    @abstractmethod
    def get_node_name(self, node):
        """Called if no explicit walker function exists for a node."""
        pass


# -----------------------------------------------------------------------------
# WalkTree
# -----------------------------------------------------------------------------


def _WalkTree(
    t: ast.AST, walk_fn: typing.Callable[[ast.AST], typing.Optional[typing.Callable]]
):
    if t and callable(walk_fn):
        walk_fn = walk_fn(t)
    if (not t) or (not callable(walk_fn)):
        return

    if isinstance(t, ast.TypeAliasStmt):
        return

    if isinstance(t, ast.ExprStmt):
        node = typing.cast(ast.ExprStmt, t)
        for x in node.exprs or []:
            _WalkTree(x, walk_fn)
        return

    if isinstance(t, ast.UnificationStmt):
        node = typing.cast(ast.UnificationStmt, t)
        if node.target:
            _WalkTree(node.target, walk_fn)
        if node.value:
            _WalkTree(node.value, walk_fn)
        return

    if isinstance(t, ast.AssignStmt):
        node = typing.cast(ast.AssignStmt, t)
        for x in node.targets or []:
            _WalkTree(x, walk_fn)
        if node.value:
            _WalkTree(node.value, walk_fn)
        return

    if isinstance(t, ast.AugAssignStmt):
        node = typing.cast(ast.AugAssignStmt, t)
        if node.target:
            _WalkTree(node.target, walk_fn)
        if node.value:
            _WalkTree(node.value, walk_fn)
        return

    if isinstance(t, ast.AssertStmt):
        node = typing.cast(ast.AssertStmt, t)
        if node.test:
            _WalkTree(node.test, walk_fn)
        if node.if_cond:
            _WalkTree(node.if_cond, walk_fn)
        if node.msg:
            _WalkTree(node.msg, walk_fn)
        return

    if isinstance(t, ast.IfStmt):
        node = typing.cast(ast.IfStmt, t)
        if node.cond:
            _WalkTree(node.cond, walk_fn)
        for x in node.body or []:
            _WalkTree(x, walk_fn)
        for x in node.elif_cond or []:
            _WalkTree(x, walk_fn)
        for if_body in node.elif_body or []:
            for x in if_body:
                _WalkTree(x, walk_fn)
        for x in node.else_body or []:
            _WalkTree(x, walk_fn)
        return

    if isinstance(t, ast.ImportStmt):
        return

    if isinstance(t, ast.SchemaIndexSignature):
        node = typing.cast(ast.SchemaIndexSignature, t)
        if node.value:
            _WalkTree(node.value, walk_fn)
        return

    if isinstance(t, ast.SchemaAttr):
        node = typing.cast(ast.SchemaAttr, t)
        if node.value:
            _WalkTree(node.value, walk_fn)
        for x in node.decorators or []:
            _WalkTree(x, walk_fn)
        return

    if isinstance(t, ast.SchemaStmt):
        node = typing.cast(ast.SchemaStmt, t)
        if node.parent_name:
            _WalkTree(node.parent_name, walk_fn)
        if node.args:
            _WalkTree(node.args, walk_fn)
        if node.for_host_name:
            _WalkTree(node.for_host_name, walk_fn)
        if node.index_signature:
            _WalkTree(node.index_signature, walk_fn)
        for x in node.mixins or []:
            _WalkTree(x, walk_fn)
        for x in node.body or []:
            _WalkTree(x, walk_fn)
        for x in node.decorators or []:
            _WalkTree(x, walk_fn)
        for x in node.checks or []:
            _WalkTree(x, walk_fn)
        return

    if isinstance(t, ast.RuleStmt):
        node = typing.cast(ast.RuleStmt, t)
        for x in node.parent_rules or []:
            _WalkTree(x, walk_fn)
        for x in node.decorators or []:
            _WalkTree(x, walk_fn)
        if node.args:
            _WalkTree(node.args, walk_fn)
        if node.for_host_name:
            _WalkTree(node.for_host_name, walk_fn)
        for x in node.checks or []:
            _WalkTree(x, walk_fn)
        return

    if isinstance(t, ast.IfExpr):
        node = typing.cast(ast.IfExpr, t)
        if node.body:
            _WalkTree(node.body, walk_fn)
        if node.cond:
            _WalkTree(node.cond, walk_fn)
        if node.orelse:
            _WalkTree(node.orelse, walk_fn)
        return

    if isinstance(t, ast.UnaryExpr):
        node = typing.cast(ast.UnaryExpr, t)
        if node.operand:
            _WalkTree(node.operand, walk_fn)
        return

    if isinstance(t, ast.BinaryExpr):
        node = typing.cast(ast.BinaryExpr, t)
        if node.left:
            _WalkTree(node.left, walk_fn)
        if node.right:
            _WalkTree(node.right, walk_fn)
        return

    if isinstance(t, ast.SelectorExpr):
        node = typing.cast(ast.SelectorExpr, t)
        if node.value:
            _WalkTree(node.value, walk_fn)
        if node.attr:
            _WalkTree(node.attr, walk_fn)
        return

    if isinstance(t, ast.CallExpr):
        node = typing.cast(ast.CallExpr, t)
        if node.func:
            _WalkTree(node.func, walk_fn)
        for x in node.args or []:
            _WalkTree(x, walk_fn)
        for x in node.keywords or []:
            _WalkTree(x, walk_fn)
        return

    if isinstance(t, ast.ParenExpr):
        node = typing.cast(ast.ParenExpr, t)
        _WalkTree(node.expr, walk_fn)
        return

    if isinstance(t, ast.QuantExpr):
        node = typing.cast(ast.QuantExpr, t)
        for var in node.variables:
            _WalkTree(var, walk_fn)
        if node.target:
            _WalkTree(node.target, walk_fn)
        if node.test:
            _WalkTree(node.test, walk_fn)
        if node.if_cond:
            _WalkTree(node.if_cond, walk_fn)
        return

    if isinstance(t, ast.ListExpr):
        node = typing.cast(ast.ListExpr, t)
        for x in node.elts or []:
            _WalkTree(x, walk_fn)
        return

    if isinstance(t, ast.ListComp):
        node = typing.cast(ast.ListComp, t)
        if node.elt:
            _WalkTree(node.elt, walk_fn)
        for x in node.generators or []:
            _WalkTree(x, walk_fn)
        return

    if isinstance(t, ast.ListIfItemExpr):
        node = typing.cast(ast.ListIfItemExpr, t)
        if node.if_cond:
            _WalkTree(node.if_cond, walk_fn)
        for expr in node.exprs or []:
            _WalkTree(expr, walk_fn)
        if node.orelse:
            _WalkTree(node.orelse, walk_fn)
        return

    if isinstance(t, ast.StarredExpr):
        node = typing.cast(ast.StarredExpr, t)
        if node.value:
            _WalkTree(node.value, walk_fn)
        return

    if isinstance(t, ast.ConfigIfEntryExpr):
        node = typing.cast(ast.ConfigIfEntryExpr, t)
        if node.if_cond:
            _WalkTree(node.if_cond, walk_fn)
        for key in node.keys or []:
            _WalkTree(key, walk_fn)
        for value in node.values or []:
            _WalkTree(value, walk_fn)
        if node.orelse:
            _WalkTree(node.orelse, walk_fn)
        return

    if isinstance(t, ast.ConfigExpr):
        node = typing.cast(ast.ConfigExpr, t)
        for x in node.keys or []:
            _WalkTree(x, walk_fn)
        for x in node.values or []:
            _WalkTree(x, walk_fn)
        return

    if isinstance(t, ast.DictComp):
        node = typing.cast(ast.DictComp, t)
        if node.key:
            _WalkTree(node.key, walk_fn)
        if node.value:
            _WalkTree(node.value, walk_fn)
        for x in node.generators or []:
            _WalkTree(x, walk_fn)
        return

    if isinstance(t, ast.CompClause):
        node = typing.cast(ast.CompClause, t)
        for x in node.targets or []:
            _WalkTree(x, walk_fn)
        if node.iter:
            _WalkTree(node.iter, walk_fn)
        for x in node.ifs or []:
            _WalkTree(x, walk_fn)
        return

    if isinstance(t, ast.SchemaExpr):
        node = typing.cast(ast.SchemaExpr, t)
        if node.name:
            _WalkTree(node.name, walk_fn)
        for x in node.args or []:
            _WalkTree(x, walk_fn)
        for x in node.kwargs or []:
            _WalkTree(x, walk_fn)
        if node.config:
            _WalkTree(node.config, walk_fn)
        return

    if isinstance(t, ast.CheckExpr):
        node = typing.cast(ast.CheckExpr, t)
        if node.test:
            _WalkTree(node.test, walk_fn)
        if node.if_cond:
            _WalkTree(node.if_cond, walk_fn)
        if node.msg:
            _WalkTree(node.msg, walk_fn)
        return

    if isinstance(t, ast.LambdaExpr):
        node = typing.cast(ast.LambdaExpr, t)
        if node.args:
            _WalkTree(node.args, walk_fn)
        for x in node.body or []:
            _WalkTree(x, walk_fn)
        return

    if isinstance(t, ast.Decorator):
        node = typing.cast(ast.Decorator, t)
        if node.name:
            _WalkTree(node.name, walk_fn)
        if node.args:
            _WalkTree(node.args, walk_fn)
        return

    if isinstance(t, ast.Subscript):
        node = typing.cast(ast.Subscript, t)
        if node.value:
            _WalkTree(node.value, walk_fn)
        if node.index:
            _WalkTree(node.index, walk_fn)
        if node.lower:
            _WalkTree(node.lower, walk_fn)
        if node.upper:
            _WalkTree(node.upper, walk_fn)
        if node.step:
            _WalkTree(node.step, walk_fn)
        return

    if isinstance(t, ast.Keyword):
        node = typing.cast(ast.Keyword, t)
        if node.arg:
            _WalkTree(node.arg, walk_fn)
        if node.value:
            _WalkTree(node.value, walk_fn)
        return

    if isinstance(t, ast.Arguments):
        node = typing.cast(ast.Arguments, t)
        for x in node.args or []:
            _WalkTree(x, walk_fn)
        for x in node.defaults or []:
            _WalkTree(x, walk_fn)
        return

    if isinstance(t, ast.Compare):
        node = typing.cast(ast.Compare, t)
        if node.left:
            _WalkTree(node.left, walk_fn)
        for x in node.comparators or []:
            _WalkTree(x, walk_fn)
        return

    if isinstance(t, ast.Identifier):
        _ = typing.cast(ast.Identifier, t)
        return

    if isinstance(t, ast.Literal):
        _ = typing.cast(ast.Literal, t)
        return
    if isinstance(t, ast.JoinedString):
        node = typing.cast(ast.JoinedString, t)
        for x in node.values or []:
            _WalkTree(x, walk_fn)
        return
    if isinstance(t, ast.FormattedValue):
        node = typing.cast(ast.FormattedValue, t)
        if node.value:
            _WalkTree(node.value, walk_fn)
        return

    if isinstance(t, ast.Module):
        node = typing.cast(ast.Module, t)
        for x in node.body or []:
            _WalkTree(x, walk_fn)
        return

    assert False, f"_WalkTree: t = {t}"


def WalkTree(m: ast.AST, walk_fn: typing.Callable[[ast.AST], None]):
    _WalkTree(m, walk_fn)
