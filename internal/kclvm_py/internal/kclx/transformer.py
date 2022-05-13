# Copyright 2021 The KCL Authors. All rights reserved.

import typing
from dataclasses import dataclass

import kclvm.config
import kclvm.kcl.error as kcl_error
import kclvm.kcl.info as kcl_info
import kclvm.kcl.ast as ast


KCLX_NODE_FIELD = "node"
BIN_OP_MAPPING = {
    ast.BinOp.Add: "Add",
    ast.BinOp.Sub: "Sub",
    ast.BinOp.Mul: "Mul",
    ast.BinOp.Div: "Div",
    ast.BinOp.Mod: "Mod",
    ast.BinOp.Pow: "Pow",
    ast.BinOp.LShift: "LShift",
    ast.BinOp.RShift: "RShift",
    ast.BinOp.BitOr: "BitOr",
    ast.BinOp.BitAnd: "BitAnd",
    ast.BinOp.BitXor: "BitXor",
    ast.BinOp.FloorDiv: "FloorDiv",
    ast.BinOp.As: "As",
    ast.BinOp.And: "And",
    ast.BinOp.Or: "Or",
}
AUG_OP_MAPPING = {
    ast.AugOp.Assign: "Assign",
    ast.AugOp.Add: "Add",
    ast.AugOp.Sub: "Sub",
    ast.AugOp.Mul: "Mul",
    ast.AugOp.Div: "Div",
    ast.AugOp.Mod: "Mod",
    ast.AugOp.Pow: "Pow",
    ast.AugOp.LShift: "LShift",
    ast.AugOp.RShift: "RShift",
    ast.AugOp.BitOr: "BitOr",
    ast.AugOp.BitXor: "BitXor",
    ast.AugOp.BitAnd: "BitAnd",
    ast.AugOp.FloorDiv: "FloorDiv",
}
UNARY_OP_MAPPING = {
    ast.UnaryOp.UAdd: "UAdd",
    ast.UnaryOp.USub: "USub",
    ast.UnaryOp.Invert: "Invert",
    ast.UnaryOp.Not: "Not",
}
CMP_OP_MAPPING = {
    ast.CmpOp.Eq: "Eq",
    ast.CmpOp.NotEq: "NotEq",
    ast.CmpOp.Lt: "Lt",
    ast.CmpOp.LtE: "LtE",
    ast.CmpOp.Gt: "Gt",
    ast.CmpOp.GtE: "GtE",
    ast.CmpOp.Is: "Is",
    ast.CmpOp.In: "In",
    ast.CmpOp.Not: "Not",
    ast.CmpOp.IsNot: "IsNot",
    ast.CmpOp.NotIn: "NotIn",
}
QUANT_OP_MAPPING = {
    ast.QuantOperation.ALL: "All",
    ast.QuantOperation.ANY: "Any",
    ast.QuantOperation.FILTER: "Filter",
    ast.QuantOperation.MAP: "Map",
}
CONFIG_ENTRY_OP_MAPPING = {
    ast.ConfigEntryOperation.UNION: "Union",
    ast.ConfigEntryOperation.OVERRIDE: "Override",
    ast.ConfigEntryOperation.INSERT: "Insert",
}
EXPR_CTX_MAPPING = {
    ast.ExprContext.LOAD: "Load",
    ast.ExprContext.STORE: "Store",
    ast.ExprContext.AUGLOAD: "Load",
    ast.ExprContext.AUGSTORE: "Store",
    ast.ExprContext.DEL: "Del",
}
OVERRIDE_ACTION_MAPPING = {
    ast.OverrideAction.CREATE_OR_UPDATE: "CreateOrUpdate",
    ast.OverrideAction.DELETE: "Delete",
}
TYPE_KCLX_ENUM_MAPPING = {
    # Stmt
    "TypeAliasStmt": "TypeAlias",
    "UnificationStmt": "Unification",
    "AssignStmt": "Assign",
    "AugAssignStmt": "AugAssign",
    "AssertStmt": "Assert",
    "IfStmt": "If",
    "ImportStmt": "Import",
    "SchemaIndexSignature": "SchemaIndexSignature",
    "SchemaAttr": "SchemaAttr",
    "SchemaStmt": "Schema",
    "RuleStmt": "Rule",
    # Expr
    "Identifier": "Identifier",
    "UnaryExpr": "Unary",
    "BinaryExpr": "Binary",
    "IfExpr": "If",
    "SelectorExpr": "Selector",
    "CallExpr": "Call",
    "ParenExpr": "Paren",
    "QuantExpr": "Quant",
    "ListExpr": "List",
    "ListIfItemExpr": "ListIfItem",
    "ListComp": "ListComp",
    "StarredExpr": "Starred",
    "DictComp": "DictComp",
    "ConfigIfEntryExpr": "ConfigIfEntry",
    "CompClause": "CompClause",
    "SchemaExpr": "Schema",
    "ConfigExpr": "Config",
    "ConfigEntry": "ConfigEntry",
    "CheckExpr": "Check",
    "LambdaExpr": "Lambda",
    "Decorator": "Decorator",
    "Subscript": "Subscript",
    "Keyword": "Keyword",
    "Arguments": "Arguments",
    "Compare": "Compare",
    "NumberLit": "NumberLit",
    "StringLit": "StringLit",
    "NameConstantLit": "NameConstantLit",
    "JoinedString": "JoinedString",
    "FormattedValue": "FormattedValue",
}
INIT_FILENAME = ""
INIT_POS = 1


@dataclass
class KCLxNode:
    filename: str
    line: int
    column: int
    end_line: int
    end_column: int


class BaseKCLxASTTransformer(ast.TreeWalker):
    @staticmethod
    def ast_meta_to_dict(t: ast.AST) -> dict:
        return KCLxNode(
            filename=t.filename or INIT_FILENAME,
            line=t.line or INIT_POS,
            column=t.column or INIT_POS,
            end_line=t.end_line or t.line or INIT_POS,
            end_column=t.end_column or t.end_column or INIT_POS,
        ).__dict__

    def get_node_name(self, t: ast.AST):
        return t.type

    def stmts(self, stmts: typing.List[ast.Stmt]):
        return [self.stmt(stmt) for stmt in stmts or []]

    def exprs(self, exprs: typing.List[ast.Expr], with_enum_name: bool = False):
        return [self.expr(expr, with_enum_name) for expr in exprs or []]

    def expr(self, expr: ast.Expr, with_enum_name: bool = False):
        expr_value = self.walk(expr) if expr else None
        if with_enum_name and expr_value and KCLX_NODE_FIELD in expr_value:
            expr_value[KCLX_NODE_FIELD] = {
                TYPE_KCLX_ENUM_MAPPING[expr._ast_type]: expr_value[KCLX_NODE_FIELD]
            }
        return expr_value

    def stmt(self, stmt: ast.Expr):
        return self.walk(stmt) if stmt else None


class KCLxASTTransformer(BaseKCLxASTTransformer):
    """TODO: Transform the Python KCL AST to the KCLx AST"""

    def walk_Module(self, t: ast.Module):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                "filename": t.filename,
                "pkg": t.pkg,
                "doc": t.doc,
                "name": t.name,
                "body": self.stmts(t.body),
                "comments": self.exprs(t.comments),
            }
        )
        return data

    def walk_TypeAliasStmt(self, t: ast.TypeAliasStmt):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    TYPE_KCLX_ENUM_MAPPING[t._ast_type]: {
                        "type_name": self.expr(t.type_name),
                        "type_value": {
                            KCLX_NODE_FIELD: t.type_value.plain_type_str,
                            **self.ast_meta_to_dict(t),
                        },
                    }
                }
            }
        )
        return data

    def walk_ExprStmt(self, t: ast.ExprStmt):
        data = self.ast_meta_to_dict(t)
        data.update({KCLX_NODE_FIELD: {"Expr": {"exprs": self.exprs(t.exprs, True)}}})
        return data

    def walk_UnificationStmt(self, t: ast.UnificationStmt):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    TYPE_KCLX_ENUM_MAPPING[t._ast_type]: {
                        "target": self.expr(t.target),
                        "value": self.expr(t.value),
                    }
                }
            }
        )
        return data

    def walk_AssignStmt(self, t: ast.AssignStmt):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    TYPE_KCLX_ENUM_MAPPING[t._ast_type]: {
                        "targets": self.exprs(t.targets),
                        "value": self.expr(t.value, True),
                        "type_annotation": {
                            KCLX_NODE_FIELD: t.type_annotation,
                            **self.ast_meta_to_dict(t),
                        }
                        if t.type_annotation
                        else None,
                    }
                }
            }
        )
        return data

    def walk_AugAssignStmt(self, t: ast.AugAssignStmt):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    TYPE_KCLX_ENUM_MAPPING[t._ast_type]: {
                        "target": self.expr(t.target),
                        "value": self.expr(t.value, True),
                        "op": AUG_OP_MAPPING[t.op],
                    }
                }
            }
        )
        return data

    def walk_AssertStmt(self, t: ast.AssertStmt):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    TYPE_KCLX_ENUM_MAPPING[t._ast_type]: {
                        "test": self.expr(t.test, True),
                        "if_cond": self.expr(t.if_cond, True),
                        "msg": self.expr(t.msg, True),
                    }
                }
            }
        )
        return data

    def walk_IfStmt(self, t: ast.IfStmt):
        data = self.ast_meta_to_dict(t)
        elif_stmt = None
        if t.elif_cond and t.elif_body:
            elif_stmt = ast.IfStmt()
            elif_stmt.set_ast_position(t)
            elif_stmt.cond = t.elif_cond[0]
            elif_stmt.body = t.elif_body[0]
            elif_stmt.elif_cond = t.elif_cond[1:]
            elif_stmt.elif_body = t.elif_body[1:]
            elif_stmt.else_body = t.else_body
        data.update(
            {
                KCLX_NODE_FIELD: {
                    TYPE_KCLX_ENUM_MAPPING[t._ast_type]: {
                        "cond": self.expr(t.cond, True),
                        "body": self.stmts(t.body),
                        "orelse": self.stmts([elif_stmt])
                        if elif_stmt
                        else self.stmts(t.else_body),
                    }
                }
            }
        )
        return data

    def walk_ImportStmt(self, t: ast.ImportStmt):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    TYPE_KCLX_ENUM_MAPPING[t._ast_type]: {
                        "path": t.path,
                        "rawpath": t.rawpath,
                        "name": t.name,
                        "asname": t.asname,
                    }
                }
            }
        )
        return data

    def walk_SchemaIndexSignature(self, t: ast.SchemaIndexSignature):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "key_name": t.key_name,
                    "value": self.expr(t.value, True),
                    "key_type": {
                        KCLX_NODE_FIELD: t.key_type,
                        **self.ast_meta_to_dict(t),
                    },
                    "value_type": {
                        KCLX_NODE_FIELD: t.value_type,
                        **self.ast_meta_to_dict(t.value_type_node),
                    },
                    "any_other": t.any_other,
                }
            }
        )
        return data

    def walk_SchemaAttr(self, t: ast.SchemaAttr):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    TYPE_KCLX_ENUM_MAPPING[t._ast_type]: {
                        "doc": t.doc,
                        "name": {
                            KCLX_NODE_FIELD: t.name,
                            **self.ast_meta_to_dict(t),
                        },
                        "type_str": {
                            KCLX_NODE_FIELD: t.type_str,
                            **self.ast_meta_to_dict(t),
                        },
                        "value": self.expr(t.value, True),
                        "op": {"Bin": BIN_OP_MAPPING[t.op]}
                        if isinstance(t.op, ast.BinOp)
                        else (
                            {"Aug": AUG_OP_MAPPING[t.op]}
                            if isinstance(t.op, ast.AugOp)
                            else None
                        ),
                        "is_optional": t.is_optional,
                        "decorators": self.exprs(t.decorators),
                    }
                }
            }
        )
        return data

    def walk_SchemaStmt(self, t: ast.SchemaStmt):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    TYPE_KCLX_ENUM_MAPPING[t._ast_type]: {
                        "doc": t.doc,
                        "name": {
                            KCLX_NODE_FIELD: t.name,
                            **self.ast_meta_to_dict(t.name_node),
                        },
                        "parent_name": self.expr(t.parent_name),
                        "for_host_name": self.expr(t.for_host_name),
                        "is_mixin": t.is_mixin,
                        "is_protocol": t.is_protocol,
                        "args": self.expr(t.args),
                        "mixins": self.exprs(t.mixins),
                        "body": self.stmts(t.body),
                        "decorators": self.exprs(t.decorators),
                        "checks": self.exprs(t.checks),
                        "index_signature": self.stmt(t.index_signature),
                    }
                }
            }
        )
        return data

    def walk_RuleStmt(self, t: ast.RuleStmt):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    TYPE_KCLX_ENUM_MAPPING[t._ast_type]: {
                        "doc": t.doc,
                        "name": {
                            KCLX_NODE_FIELD: t.name,
                            **self.ast_meta_to_dict(t.name_node),
                        },
                        "parent_rules": self.exprs(t.parent_rules),
                        "for_host_name": self.expr(t.for_host_name),
                        "args": self.expr(t.args),
                        "decorators": self.exprs(t.decorators),
                        "checks": self.exprs(t.checks),
                    }
                }
            }
        )
        return data

    def walk_Identifier(self, t: ast.Identifier):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "names": t.names,
                    "pkgpath": t.names[0]
                    if t.names[0].startswith("@")
                    else (t.pkgpath or ""),
                    "ctx": EXPR_CTX_MAPPING.get(t.ctx, "Load"),
                }
            }
        )
        return data

    def walk_UnaryExpr(self, t: ast.UnaryExpr):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "op": UNARY_OP_MAPPING[t.op],
                    "operand": self.expr(t.operand, True),
                }
            }
        )
        return data

    def walk_BinaryExpr(self, t: ast.BinaryExpr):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "left": self.expr(t.left, True),
                    "op": {"Bin": BIN_OP_MAPPING[t.op]}
                    if isinstance(t.op, ast.BinOp)
                    else {"Cmp": CMP_OP_MAPPING[t.op]},
                    "right": self.expr(t.right, True),
                }
            }
        )
        return data

    def walk_IfExpr(self, t: ast.IfExpr):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "body": self.expr(t.body, True),
                    "cond": self.expr(t.cond, True),
                    "orelse": self.expr(t.orelse, True),
                }
            }
        )
        return data

    def walk_SelectorExpr(self, t: ast.SelectorExpr):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "value": self.expr(t.value, True),
                    "attr": self.expr(t.attr),
                    "ctx": EXPR_CTX_MAPPING[t.ctx],
                    "has_question": t.has_question,
                }
            }
        )
        return data

    def walk_CallExpr(self, t: ast.CallExpr):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "func": self.expr(t.func, True),
                    "args": self.exprs(t.args, True),
                    "keywords": self.exprs(t.keywords),
                }
            }
        )
        return data

    def walk_ParenExpr(self, t: ast.ParenExpr):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "expr": self.expr(t.expr, True),
                }
            }
        )
        return data

    def walk_QuantExpr(self, t: ast.QuantExpr):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "target": self.expr(t.target, True),
                    "variables": self.exprs(t.variables),
                    "op": QUANT_OP_MAPPING[t.op],
                    "test": self.expr(t.test, True),
                    "if_cond": self.expr(t.if_cond, True),
                    "ctx": EXPR_CTX_MAPPING[t.ctx],
                }
            }
        )
        return data

    def walk_ListExpr(self, t: ast.ListExpr):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "elts": self.exprs(t.elts, True),
                    "ctx": EXPR_CTX_MAPPING[t.ctx],
                }
            }
        )
        return data

    def walk_ListIfItemExpr(self, t: ast.ListIfItemExpr):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "if_cond": self.expr(t.if_cond, True),
                    "exprs": self.exprs(t.exprs, True),
                    "orelse": self.expr(t.orelse, True),
                }
            }
        )
        return data

    def walk_ListComp(self, t: ast.ListComp):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "elt": self.expr(t.elt, True),
                    "generators": self.exprs(t.generators),
                }
            }
        )
        return data

    def walk_StarredExpr(self, t: ast.StarredExpr):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "value": self.expr(t.value, True),
                    "ctx": EXPR_CTX_MAPPING[t.ctx],
                }
            }
        )
        return data

    def walk_DictComp(self, t: ast.DictComp):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "entry": {
                        "key": self.expr(t.key, True),
                        "value": self.expr(t.value, True),
                        "operation": CONFIG_ENTRY_OP_MAPPING[t.operation],
                        "insert_index": -1,
                    },
                    "generators": self.exprs(t.generators),
                }
            }
        )
        return data

    def walk_ConfigIfEntryExpr(self, t: ast.ConfigIfEntryExpr):
        data = self.ast_meta_to_dict(t)
        keys = []
        values = []
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
            keys.append(key)
            values.append(value)
        t.keys = keys
        t.values = values
        items = []
        for key, value, op in zip(t.keys, t.values, t.operations):
            items.append(
                ast.ConfigEntry(
                    line=key.line if key else value.line,
                    column=key.column if key else value.column,
                    key=key,
                    value=value,
                    operation=op,
                )
            )
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "if_cond": self.expr(t.if_cond, True),
                    "items": self.exprs(items),
                    "orelse": self.expr(t.orelse, True),
                }
            }
        )
        return data

    def walk_CompClause(self, t: ast.CompClause):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "targets": self.exprs(t.targets),
                    "iter": self.expr(t.iter, True),
                    "ifs": self.exprs(t.ifs, True),
                }
            }
        )
        return data

    def walk_SchemaExpr(self, t: ast.SchemaExpr):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "name": self.expr(t.name),
                    "args": self.exprs(t.args, True),
                    "kwargs": self.exprs(t.kwargs),
                    "config": self.expr(t.config, True),
                }
            }
        )
        return data

    def walk_ConfigExpr(self, t: ast.ConfigExpr):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "items": self.exprs(t.items),
                }
            }
        )
        return data

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
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "key": self.expr(t.key, True),
                    "value": self.expr(t.value, True),
                    "operation": CONFIG_ENTRY_OP_MAPPING[
                        ast.ConfigEntryOperation.UNION if is_nest_key else t.operation
                    ],
                    "insert_index": t.insert_index,
                }
            }
        )
        return data

    def walk_CheckExpr(self, t: ast.CheckExpr):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "test": self.expr(t.test, True),
                    "if_cond": self.expr(t.if_cond, True),
                    "msg": self.expr(t.msg, True),
                }
            }
        )
        return data

    def walk_LambdaExpr(self, t: ast.LambdaExpr):
        """ast.AST: LambdaExpr

        Parameters
        ----------
        - args: Optional[Arguments]
        - return_type_str: Optional[str]
        - return_type_node: Optional[Type]
        - body: List[Stmt]
        """
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "args": self.expr(t.args),
                    "return_type_str": t.return_type_str,
                    "body": self.stmts(t.body),
                }
            }
        )
        return data

    def walk_Decorator(self, t: ast.Decorator):
        name = self.expr(t.name, True)
        call_expr = self.expr(t.args)
        if call_expr:
            call_expr[KCLX_NODE_FIELD]["func"] = name
            return call_expr
        else:
            data = self.ast_meta_to_dict(t)
            data.update(
                {
                    KCLX_NODE_FIELD: {
                        "func": name,
                        "args": [],
                        "keywords": [],
                    }
                }
            )
            return data

    def walk_Subscript(self, t: ast.Subscript):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "value": self.expr(t.value, True),
                    "index": self.expr(t.index, True),
                    "lower": self.expr(t.lower, True),
                    "upper": self.expr(t.upper, True),
                    "step": self.expr(t.step, True),
                    "ctx": EXPR_CTX_MAPPING[t.ctx],
                    "has_question": t.has_question,
                }
            }
        )
        return data

    def walk_Keyword(self, t: ast.Keyword):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "arg": self.expr(t.arg),
                    "value": self.expr(t.value, True),
                }
            }
        )
        return data

    def walk_Arguments(self, t: ast.Arguments):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "args": self.exprs(t.args),
                    "defaults": self.exprs(t.defaults, True),
                    "type_annotation_list": [
                        {KCLX_NODE_FIELD: tpe_str, **self.ast_meta_to_dict(t)}
                        if tpe_str
                        else None
                        for tpe_str, tpe_node in zip(
                            t.type_annotation_list, t.type_annotation_node_list
                        )
                    ],
                }
            }
        )
        return data

    def walk_Compare(self, t: ast.Compare):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "left": self.expr(t.left, True),
                    "ops": [CMP_OP_MAPPING[op] for op in t.ops],
                    "comparators": self.exprs(t.comparators, True),
                }
            }
        )
        return data

    def walk_JoinedString(self, t: ast.JoinedString):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "is_long_string": t.is_long_string,
                    "values": self.exprs(t.values, True),
                    "raw_value": t.raw_value,
                }
            }
        )
        return data

    def walk_FormattedValue(self, t: ast.FormattedValue):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "is_long_string": t.is_long_string,
                    "value": self.expr(t.value, True),
                    "format_spec": t.format_spec,
                }
            }
        )
        return data

    def walk_NumberLit(self, t: ast.NumberLit):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "binary_suffix": t.binary_suffix,
                    "value": {"Int": t.value}
                    if isinstance(t.value, int)
                    else {"Float": t.value},
                }
            }
        )
        return data

    def walk_StringLit(self, t: ast.StringLit):
        data = self.ast_meta_to_dict(t)
        data.update(
            {
                KCLX_NODE_FIELD: {
                    "is_long_string": t.is_long_string,
                    "raw_value": t.raw_value or t.value,
                    "value": t.value,
                }
            }
        )
        return data

    def walk_NameConstantLit(self, t: ast.NameConstantLit):
        data = self.ast_meta_to_dict(t)
        value = "Undefined"
        value = "None" if t.value is None else value
        value = "True" if t.value is True else value
        value = "False" if t.value is False else value
        data.update({KCLX_NODE_FIELD: {"value": value}})
        return data

    def walk_Comment(self, t: ast.Comment):
        data = self.ast_meta_to_dict(t)
        data.update({KCLX_NODE_FIELD: {"text": t.text}})
        return data


def transform_ast_to_kclx_ast_json_str(program: ast.Program) -> str:
    check_number_lit_range(program)
    for pkgpath in program.pkgs:
        for i, module in enumerate(program.pkgs[pkgpath]):
            program.pkgs[pkgpath][i] = KCLxASTTransformer().walk_Module(module)
    return program.to_json(indent=None)


def check_number_lit_range(program: ast.Program):
    strict_range_check = kclvm.config.strict_range_check
    check_bit = 32 if strict_range_check else 64
    int_min = kcl_info.INT32_MIN if strict_range_check else kcl_info.INT64_MIN
    int_max = kcl_info.INT32_MAX if strict_range_check else kcl_info.INT64_MAX
    float_min = kcl_info.FLOAT32_MIN if strict_range_check else kcl_info.FLOAT64_MIN
    float_max = kcl_info.FLOAT32_MAX if strict_range_check else kcl_info.FLOAT64_MAX

    def walk_lit(t: ast.AST) -> typing.Optional[typing.Callable]:
        if isinstance(t, (ast.NumberLit)):
            numberLit = typing.cast(ast.NumberLit, t)
            value = numberLit.value

            if isinstance(value, int):
                if not (int_min <= value <= int_max):
                    kcl_error.report_exception(
                        err_type=kcl_error.ErrType.IntOverflow_TYPE,
                        file_msgs=[
                            kcl_error.ErrFileMsg(
                                filename=numberLit.filename, line_no=numberLit.line
                            )
                        ],
                        arg_msg=kcl_error.INT_OVER_FLOW_MSG.format(
                            str(value), check_bit
                        ),
                    )
            elif isinstance(value, float):
                abs_var = abs(value)
                if 0 < abs_var < float_min:
                    kcl_error.report_exception(
                        err_type=kcl_error.ErrType.FloatUnderflow_TYPE,
                        file_msgs=[
                            kcl_error.ErrFileMsg(
                                filename=numberLit.filename, line_no=numberLit.line
                            )
                        ],
                        arg_msg=kcl_error.FLOAT_UNDER_FLOW_MSG.format(
                            str(value), check_bit
                        ),
                    )
                elif abs_var > float_max:
                    kcl_error.report_exception(
                        err_type=kcl_error.ErrType.FloatOverflow_TYPE,
                        file_msgs=[
                            kcl_error.ErrFileMsg(
                                filename=numberLit.filename, line_no=numberLit.line
                            )
                        ],
                        arg_msg=kcl_error.FLOAT_OVER_FLOW_MSG.format(
                            str(value), check_bit
                        ),
                    )

        return walk_lit

    for pkgpath in program.pkgs:
        for i, module in enumerate(program.pkgs[pkgpath]):
            ast.WalkTree(module, walk_lit)
