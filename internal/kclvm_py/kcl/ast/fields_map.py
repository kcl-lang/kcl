# Copyright 2020 The KCL Authors. All rights reserved.

from typing import Tuple, Union, List

import kclvm.kcl.ast as ast

# Auto Generated
AST_FIELDS_MAP = {
    # Stmt
    "ExprStmt": ["exprs"],
    "UnificationStmt": ["target", "value"],
    "TypeAliasStmt": ["type_name"],
    "AssignStmt": ["targets", "value", "type_annotation_node"],
    "AugAssignStmt": ["target", "op", "value"],
    "AssertStmt": ["test", "if_cond", "msg"],
    "IfStmt": ["cond", "body", "elif_cond", "elif_body", "else_body"],
    "ImportStmt": ["path", "name", "asname", "rawpath", "path_nodes", "as_name_node"],
    "SchemaAttr": [
        "doc",
        "name",
        "type_str",
        "op",
        "value",
        "is_optional",
        "decorators",
        "name_node",
        "type_node",
    ],
    "SchemaStmt": [
        "doc",
        "name",
        "parent_name",
        "is_mixin",
        "args",
        "mixins",
        "body",
        "decorators",
        "checks",
        "index_signature",
        "name_node",
        "for_host_name",
    ],
    "RuleStmt": [
        "doc",
        "name",
        "parent_rules",
        "decorators",
        "args",
        "checks",
        "name_node",
        "for_host_name",
    ],
    # Expr
    "QuantExpr": ["target", "variables", "test", "if_cond"],
    "SchemaIndexSignature": ["value", "name_node", "value_type_node"],
    "IfExpr": ["body", "cond", "orelse"],
    "UnaryExpr": ["op", "operand"],
    "BinaryExpr": ["left", "op", "right"],
    "SelectorExpr": ["value", "attr", "has_question"],
    "CallExpr": ["func", "args", "keywords"],
    "Subscript": ["value", "index", "lower", "upper", "step", "has_question"],
    "ParenExpr": ["expr"],
    "Operand": ["value"],
    "ListExpr": ["elts"],
    "ListComp": ["elt", "generators"],
    "ListIfItemExpr": ["if_cond", "exprs", "orelse"],
    "StarredExpr": ["value"],
    "DictComp": ["key", "value", "generators"],
    "ConfigIfEntryExpr": ["if_cond", "keys", "values", "orelse"],
    "CompClause": ["targets", "iter", "ifs"],
    "SchemaExpr": ["name", "args", "kwargs", "config"],
    "ConfigExpr": ["items"],
    "ConfigEntry": ["key", "value", "operation", "insert_index"],
    "CheckExpr": ["test", "if_cond", "msg"],
    "LambdaExpr": ["args", "body"],
    "Decorator": ["name", "args"],
    "Keyword": ["arg", "value"],
    "Arguments": ["args", "defaults", "type_annotation_node_list"],
    "Compare": ["left", "ops", "comparators"],
    "Identifier": ["names", "name_nodes"],
    "Name": ["value"],
    "Literal": ["value"],
    "NumberLit": ["value"],
    "StringLit": ["value", "is_long_string"],
    "NameConstantLit": ["value"],
    "JoinedString": ["values", "is_long_string"],
    "FormattedValue": ["value", "is_long_string", "format_spec"],
    "Comment": ["text"],
    "CommentGroup": ["comments"],
    "Module": ["body", "comments"],
    # Type
    "Type": ["type_elements"],
    "BasicType": ["type_name"],
    "ListType": ["inner_type"],
    "DictType": ["key_type", "value_type"],
    "LiteralType": ["string_value", "number_value"],
}


def iter_fields(t: ast.AST) -> List[Tuple[str, Union[ast.AST, List[ast.AST]]]]:
    """Return ast node attribute-value AST pair"""
    if not t or not isinstance(t, ast.AST):
        return []
    return [(field, getattr(t, field)) for field in AST_FIELDS_MAP[t.type]]
