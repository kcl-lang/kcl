# Copyright 2021 The KCL Authors. All rights reserved.

from typing import cast, List, Union, Optional, Type
from dataclasses import dataclass

import kclvm.kcl.ast as ast


@dataclass
class Declaration:
    filename: str
    name: str
    value: ast.Expr
    is_union: bool


def filter_declarations(
    module: ast.Module,
    ast_type: Optional[Union[Type[ast.AST], str, tuple, list]] = None,
) -> List[Declaration]:
    """Get all global AssignStmt key-value pair config according to the `ast_type`.
    When the `ast_type` is None, select all declarations
    """
    if not module or not isinstance(module, ast.Module):
        return []
    declaration_list = []
    for stmt in module.body or []:
        declaration = None
        if isinstance(stmt, ast.AssignStmt):
            stmt = cast(ast.AssignStmt, stmt)
            for target in stmt.targets:
                name = target.get_name()
                if target.ctx == ast.ExprContext.STORE:
                    value = cast(ast.Expr, stmt.value)
                    declaration = Declaration(
                        filename=stmt.filename,
                        name=name,
                        value=value,
                        is_union=False,
                    )
        elif isinstance(stmt, ast.UnificationStmt):
            stmt = cast(ast.UnificationStmt, stmt)
            name = stmt.target.get_name()
            value = cast(ast.Expr, stmt.value)
            declaration = Declaration(
                filename=stmt.filename, name=name, value=stmt.value, is_union=True
            )
        if declaration:
            if ast_type is None:
                declaration_list.append(declaration)
            elif isinstance(ast_type, (list, tuple)) and isinstance(
                stmt.value, tuple(ast_type)
            ):
                declaration_list.append(declaration)
            elif isinstance(ast_type, str) and value.type == ast_type:
                declaration_list.append(declaration)
            elif isinstance(ast_type, type(ast.AST)) and isinstance(
                stmt.value, ast_type
            ):
                declaration_list.append(declaration)
    return declaration_list


def filter_stmt(
    module: ast.Module, stmt_type: Union[str, Type[ast.Stmt]]
) -> List[ast.Stmt]:
    """Get all AugAssignStmt at the top level of the module"""
    if not module or not isinstance(module, ast.Module):
        return []
    if not stmt_type:
        return []
    result = []
    for stmt in module.body or []:
        if stmt.type == stmt_type or isinstance(stmt, stmt_type):
            result.append(stmt)
    return result
