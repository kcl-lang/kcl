# Copyright 2021 The KCL Authors. All rights reserved.

import io
from dataclasses import dataclass
from typing import List

import kclvm.kcl.ast as ast
import kclvm.compiler.parser as parser

from .printer import PrintAST


NEWLINE = "\n"
MOCK_SCHEMA = "MockSchema"


@dataclass
class SchemaRuleCodeSnippet:
    """Schema and rule code snippet structure"""

    schema: str = ""
    rule: str = ""


def splice_schema_with_rule(code_snippets: List[SchemaRuleCodeSnippet]) -> str:
    """Splice schema with rule code

    Returns a string result denoting the splicing code.

    Parameters
    ----------
    code_snippets : List[SchemaRuleCodeSnippet]
        A list of schema and rule code snippet structure
    """
    if not isinstance(code_snippets, list):
        raise ValueError(f"Invalid parameter {code_snippets}, expected list")
    with io.StringIO() as buf:
        for code in code_snippets:
            if not code or not isinstance(code, SchemaRuleCodeSnippet):
                raise ValueError(
                    f"Invalid parameter {code}, expected SchemaRuleCodeSnippet"
                )
            module = parser.ParseFile(
                "<schema>.k", code.schema, mode=parser.ParseMode.ParseComments
            )
            module_schema_rule = parser.ParseFile(
                "<rule>.k",
                build_rule_check_block_str(
                    MOCK_SCHEMA,
                    code.rule,
                ),
                mode=parser.ParseMode.ParseComments,
            )
            schema_rule_list = module_schema_rule.GetSchemaList()
            if schema_rule_list:
                for i, stmt in enumerate(module.body):
                    if isinstance(module.body[i], ast.SchemaStmt):
                        module.body[i].checks = schema_rule_list[0].checks
                        for comment in module_schema_rule.comments:
                            comment.line = module.body[i].end_line + 1
                            module.comments.append(comment)
            PrintAST(module, buf)
        return buf.getvalue().rstrip(NEWLINE) + NEWLINE


def add_indent_to_code_string(code: str, indent: int = 4) -> str:
    """Add indent to code string"""
    if not code or not isinstance(code, str):
        return ""
    lines = code.split(NEWLINE)
    return NEWLINE.join([" " * indent + line for line in lines])


def build_rule_check_block_str(schema_name: str, rule_code: str) -> str:
    """Build rule check block string using the rule code string"""
    if not schema_name or not isinstance(rule_code, str):
        return ""
    if not rule_code or not isinstance(rule_code, str):
        return ""
    return (
        f"schema {schema_name}:"
        + NEWLINE
        + add_indent_to_code_string("check:", 4)
        + NEWLINE
        + add_indent_to_code_string(rule_code, 8)
    )
