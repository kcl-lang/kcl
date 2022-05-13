# Copyright 2021 The KCL Authors. All rights reserved.

from typing import Optional

import kclvm.kcl.ast as ast

_IDENTIFIER_PREFIX = "$"


class ASTIdentifierPrefixTransformer(ast.TreeTransformer):
    @staticmethod
    def _remove_prefix(name: str):
        if not name:
            return name
        return name.replace(_IDENTIFIER_PREFIX, "")

    def walk_Name(self, node: ast.Name) -> None:
        node.value = self._remove_prefix(node.value)
        return node

    def walk_Identifier(self, node: ast.Identifier):
        node.names = [self._remove_prefix(name) for name in node.names]
        for name_node in node.name_nodes or []:
            self.walk(name_node)
        return node

    def walk_SchemaStmt(self, node: ast.SchemaStmt):
        node.name = self._remove_prefix(node.name)
        if node.parent_name:
            node.parent_name = self.walk(node.parent_name)
        for mixin in node.mixins or []:
            self.walk(mixin)
        for stmt in node.body or []:
            self.walk(stmt)
        for check in node.checks or []:
            self.walk(check)
        return node

    def walk_RuleStmt(self, node: ast.RuleStmt):
        node.name = self._remove_prefix(node.name)
        for rule in node.parent_rules or []:
            self.walk(rule)
        for check in node.checks or []:
            self.walk(check)
        return node

    def walk_SchemaAttr(self, node: ast.SchemaAttr):
        node.name = self._remove_prefix(node.name)
        node.type_str = self._remove_prefix(node.type_str)
        return node

    def walk_ImportStmt(self, node: ast.ImportStmt):
        if node.asname:
            node.asname = self._remove_prefix(node.asname)
        if node.as_name_node:
            self.walk(node.as_name_node)
        node.name = self._remove_prefix(node.name)
        node.path = self._remove_prefix(node.path)
        for path_node in node.path_nodes or []:
            self.walk(path_node)
        return node


class ConfigNestVarTransformer(ast.TreeTransformer):
    def walk_ConfigEntry(self, t: ast.ConfigEntry):
        # Unpack the nest var form `a.b.c = 1` to `a: {b: {c = 1}}`
        is_nest_key = isinstance(t.key, ast.Identifier) and len(t.key.names) > 1
        if is_nest_key:
            names = t.key.names
            value = t.value
            t.key.names = [t.key.names[0]]
            for i, name in enumerate(names[1:][::-1]):
                is_last_item = i == 0
                name_node = ast.Identifier(
                    names=[name], line=t.key.line, column=t.key.column
                )
                name_node.filename = t.filename
                entry_value = ast.ASTFactory.get_ast_configentry(
                    name_node,
                    value,
                    t.operation if is_last_item else ast.ConfigEntryOperation.UNION,
                    t.filename,
                )
                value = ast.ConfigExpr(line=t.key.line, column=t.key.column)
                value.filename = t.filename
                value.items.append(entry_value)
            t.value = value
            t.operation = ast.ConfigEntryOperation.UNION
        self.walk(t.value)
        return t

    def walk_ConfigIfEntryExpr(self, t: ast.ConfigIfEntryExpr):
        keys = []
        values = []
        operations = []
        for key, value, operation in zip(t.keys, t.values, t.operations):
            is_nest_key = isinstance(key, ast.Identifier) and len(key.names) > 1
            if is_nest_key:
                names = key.names
                key.names = [key.names[0]]
                for i, name in enumerate(names[1:][::-1]):
                    is_last_item = i == 0
                    name_node = ast.Identifier(
                        names=[name], line=key.line, column=key.column
                    )
                    name_node.filename = t.filename
                    entry_value = ast.ASTFactory.get_ast_configentry(
                        name_node,
                        value,
                        operation if is_last_item else ast.ConfigEntryOperation.UNION,
                        t.filename,
                    )
                    value = ast.ConfigExpr(line=key.line, column=key.column)
                    value.filename = t.filename
                    value.items.append(entry_value)
                operations.append(ast.ConfigEntryOperation.UNION)
            else:
                operations.append(operation)
            keys.append(key)
            values.append(value)
        t.keys = keys
        t.values = [self.walk(v) for v in values]
        t.operations = operations
        if t.orelse:
            self.walk(t.orelse)
        return t


def fix_identifier_prefix(node: Optional[ast.AST]) -> Optional[ast.AST]:
    """Fix AST Identifier prefix and unpack the nest var form `a.b.c = 1` to `a: {b: {c = 1}}`

    Examples
    --------
    $filter -> filter
    """
    if not node or not isinstance(node, ast.AST):
        return node
    ConfigNestVarTransformer().walk(node)
    return ASTIdentifierPrefixTransformer().walk(node)
