# Copyright 2021 The KCL Authors. All rights reserved.

from dataclasses import dataclass
from abc import ABC, abstractmethod
from copy import deepcopy
from enum import IntEnum
from collections import defaultdict
from typing import Tuple, List, Optional, cast

import kclvm.kcl.ast as ast
import kclvm.compiler.astutil as astutil

from .vertex import Vertex
from .unifier import Unifier, UnifierConfig


INVALID_STRATEGY_MSG = "invalid strategy {}"


class BaseMerger(ABC):
    def __init__(self, module: ast.Module):
        self.module: ast.Module = module

    @abstractmethod
    def merge(self) -> Tuple[Vertex, Vertex, ast.Module]:
        raise NotImplementedError


@dataclass
class UnifyMerger(BaseMerger):
    def __init__(self, module: ast.Module):
        super().__init__(module)

    def unify_ast_module(
        self, config: UnifierConfig = UnifierConfig()
    ) -> Tuple[Optional[Vertex], Optional[Vertex], ast.Module]:
        """Unify vertex according to the AST Module"""
        if not self.module:
            return None, None, self.module
        vertex = Vertex.ast_to_vertex(self.module)
        unify_vertex = Unifier(config).unify(vertex)
        merged_module = cast(ast.Module, unify_vertex.vertex_to_ast())
        return vertex, unify_vertex, merged_module

    def merge(self) -> Tuple[Vertex, Vertex, ast.Module]:
        """Merge the AST Module

        Input
        -----
            None

        Output
        ------
            vertex: Vertex formed by AST before merging
            unify_vertex: Vertex formed by AST after merging
            module: The merged AST
        """
        vertex, unify_vertex, merged_module = self.unify_ast_module()
        self.module = _deal_origin_ast_module(merged_module, self.module)
        return vertex, unify_vertex, self.module


@dataclass
class UniqueUnifyMerger(UnifyMerger):
    def __init__(self, module: ast.Module):
        super().__init__(module)

    def merge(self) -> Tuple[Vertex, Vertex, ast.Module]:
        """Merge the AST Module

        Input
        -----
            None

        Output
        ------
            vertex: Vertex formed by AST before merging
            unify_vertex: Vertex formed by AST after merging
            module: The merged AST
        """
        vertex, unify_vertex, merged_module = self.unify_ast_module(
            config=UnifierConfig(check_unique=True)
        )
        self.module = _deal_origin_ast_module(merged_module, self.module)
        return vertex, unify_vertex, self.module


@dataclass
class Overrider(UnifyMerger):
    def __init__(self, module: ast.Module):
        super().__init__(module)

    def merge(self) -> Tuple[Vertex, Vertex, ast.Module]:
        vertex, unify_vertex, merged_module = self.unify_ast_module(
            config=UnifierConfig(override=True)
        )
        self.module = _deal_origin_ast_module(merged_module, self.module)
        return vertex, unify_vertex, self.module


class MergeStrategy(IntEnum):
    UNION = 1  # `:`
    OVERRIDE = 2  # `=`
    UNIQUE = 3  # `!`


class MergeStrategyFactory:

    MAPPING = {
        MergeStrategy.UNION: UnifyMerger,
        MergeStrategy.OVERRIDE: Overrider,
        MergeStrategy.UNIQUE: UniqueUnifyMerger,
    }

    @staticmethod
    def get(strategy: MergeStrategy):
        return MergeStrategyFactory.MAPPING.get(strategy, UnifyMerger)


def _deal_origin_ast_module(
    merged_module: ast.Module, origin_module: ast.Module, all_visited: bool = False
) -> Optional[ast.Module]:
    """Deal origin AST Module according to merged_module"""
    if not merged_module or not isinstance(merged_module, ast.Module):
        return None
    merged_declaration_list = astutil.filter_declarations(merged_module, ast.SchemaExpr)
    merged_declaration_map = defaultdict(list)
    for d in merged_declaration_list:
        d.value.filename = d.filename
        merged_declaration_map[d.name].append(d.value)
    merged_declaration_visited = {d.name: False for d in merged_declaration_list}
    # Reverse traversal
    origin_module.body = origin_module.body[::-1]
    i = 0
    # TODO: Optimize the while loop using AST Transformer.
    while i < len(origin_module.body):
        stmt = origin_module.body[i]
        if isinstance(stmt, ast.UnificationStmt):
            target = stmt.target
            name = target.get_first_name()
            if name in merged_declaration_map:
                del origin_module.body[i]
                i -= 1
                values = merged_declaration_map[name]
                if values and not merged_declaration_visited[name] and not all_visited:
                    identifier = ast.Identifier(
                        line=target.line,
                        column=target.column,
                        names=[name],
                        ctx=ast.ExprContext.STORE,
                    )
                    identifier.pkgpath = target.pkgpath
                    identifier.filename = target.filename
                    identifier.end_line, identifier.end_column = (
                        target.end_line,
                        target.end_column,
                    )
                    insert_unification_stmt = ast.UnificationStmt(
                        line=stmt.line, column=stmt.column
                    )
                    insert_unification_stmt.target = identifier
                    (
                        insert_unification_stmt.end_line,
                        insert_unification_stmt.end_column,
                    ) = (
                        stmt.end_line,
                        stmt.end_column,
                    )
                    for value in values:
                        stmt_copy = deepcopy(insert_unification_stmt)
                        stmt_copy.value = value
                        stmt_copy.filename = value.filename
                        stmt_copy.line = value.line
                        stmt_copy.column = value.column
                        stmt_copy.end_line = value.end_line
                        stmt_copy.end_column = value.end_column
                        i += 1
                        origin_module.body.insert(i, stmt_copy)
                    merged_declaration_visited[name] = True
        elif isinstance(stmt, ast.AssignStmt) and isinstance(
            stmt.value, ast.SchemaExpr
        ):
            if not stmt.targets:
                del origin_module.body[i]
                i -= 1
            j = 0
            while j < len(stmt.targets):
                target = stmt.targets[j]
                name = target.get_first_name()
                if name in merged_declaration_map:
                    del stmt.targets[j]
                    j -= 1
                    if len(stmt.targets) == 0:
                        del origin_module.body[i]
                        i -= 1
                    values = merged_declaration_map[name]
                    if (
                        values
                        and not merged_declaration_visited[name]
                        and not all_visited
                    ):
                        identifier = ast.Identifier(
                            line=target.line,
                            column=target.column,
                            names=[name],
                            ctx=ast.ExprContext.STORE,
                        )
                        identifier.pkgpath = target.pkgpath
                        identifier.filename = target.filename
                        identifier.end_line, identifier.end_column = (
                            target.end_line,
                            target.end_column,
                        )
                        insert_assign_stmt = ast.AssignStmt(
                            line=stmt.line, column=stmt.column
                        )
                        insert_assign_stmt.targets = [identifier]
                        insert_assign_stmt.end_line, insert_assign_stmt.end_column = (
                            stmt.end_line,
                            stmt.end_column,
                        )
                        for value in values:
                            stmt_copy = deepcopy(insert_assign_stmt)
                            stmt_copy.value = value
                            stmt_copy.filename = value.filename
                            stmt_copy.line = value.line
                            stmt_copy.column = value.column
                            stmt_copy.end_line = value.end_line
                            stmt_copy.end_column = value.end_column
                            i += 1
                            origin_module.body.insert(i, stmt_copy)
                        merged_declaration_visited[name] = True
                j += 1
        i += 1
    # Remove empty targets assignment
    origin_module.body = [
        m
        for m in reversed(origin_module.body)
        if not isinstance(m, ast.AssignStmt) or m.targets
    ]
    return origin_module


def MergeASTList(
    modules: List[ast.Module], strategy: MergeStrategy = MergeStrategy.UNION
) -> List[ast.Module]:
    """Merge the configurations of the same name in the
    AST Module list, and the ones that cannot be merged
    will be handed over to the VM for calculation.
    """
    if not modules or not isinstance(modules, list):
        return []
    # AST module filename list
    filenames = [m.filename for m in modules]
    # Config need to be merged
    file_configs = [
        stmt
        for m in modules[1:]
        for stmt in m.body
        if (isinstance(stmt, ast.AssignStmt) and isinstance(stmt.value, ast.SchemaExpr))
        or isinstance(stmt, ast.UnificationStmt)
        or isinstance(stmt, ast.ImportStmt)
    ]
    # Config filename meta
    files_meta = [
        (m.filename, stmt.line, stmt.column, stmt.end_line, stmt.end_column)
        for m in modules[1:]
        for stmt in m.body
        if (isinstance(stmt, ast.AssignStmt) and isinstance(stmt.value, ast.SchemaExpr))
        or isinstance(stmt, ast.UnificationStmt)
        or isinstance(stmt, ast.ImportStmt)
    ]
    # Record the statement filename
    for i, config in enumerate(file_configs):
        (
            file_configs[i].filename,
            file_configs[i].line,
            file_configs[i].column,
            file_configs[i].end_line,
            file_configs[i].end_column,
        ) = files_meta[i]
    if not modules[0].body:
        modules[0].body = []
    modules[0].body += file_configs
    MergeAST(modules[0], strategy)
    # Other file config list
    file_configs = [
        stmt
        for stmt in modules[0].body
        if stmt.filename is not None
        and stmt.filename != modules[0].filename
        and not isinstance(stmt, ast.ImportStmt)
    ]
    # Origin module
    modules[0].body = [stmt for stmt in modules[0].body if stmt not in file_configs]
    # Filter all config except the first file
    for i, _ in enumerate(modules[1:]):
        modules[i + 1].body = [
            stmt
            for stmt in modules[i + 1].body
            if (
                not isinstance(stmt, ast.AssignStmt)
                or not isinstance(stmt.value, ast.SchemaExpr)
            )
            and not isinstance(stmt, ast.UnificationStmt)
        ]
    # Insert the merged configuration into different files
    for config in file_configs:
        index = filenames.index(config.filename)
        line = config.line
        insert_index = len(modules[index].body)
        for i, stmt in enumerate(modules[index].body):
            if stmt.line >= line:
                insert_index = i
                break
        modules[index].body.insert(insert_index, config)
    # Return the merged multi-file modules
    return modules


def MergeAST(
    module: ast.Module, strategy: MergeStrategy = MergeStrategy.UNION
) -> ast.Module:
    """Merge the configurations of the same name in the single
    AST module, and the ones that cannot be merged will be handed
    over to the VM for calculation.
    """
    _, _, merged_module = MergeASTToVertex(module, strategy)
    return merged_module


def MergeASTToVertex(
    module: ast.Module, strategy: MergeStrategy = MergeStrategy.UNION
) -> Tuple[Optional[Vertex], Optional[Vertex], ast.Module]:
    """Merge the configurations of the same name in the AST and
    return the merged vertices, and the ones that cannot be merged
    will be handed over to the VM for calculation.
    """
    if not module:
        return None, None, module
    merger = MergeStrategyFactory.get(strategy)
    return merger(module).merge()
