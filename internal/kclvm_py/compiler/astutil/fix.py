# Copyright 2021 The KCL Authors. All rights reserved.

import re
import typing
import copy
from collections import OrderedDict

import kclvm.kcl.error as kcl_error
import kclvm.kcl.info as kcl_info
import kclvm.kcl.ast as ast
import kclvm.compiler.parser.lark_parser as lark_parser
import kclvm.compiler.vfs as vfs

PKGPATH_IDENTIFIER_DOT_REGEX = r"[\d\w_]+\."
PKGPATH_DOT_REGEX = r"@[\d\w_\.]+\."


def _get_global_names(m: ast.Module) -> typing.List[str]:
    assert m
    assert isinstance(m, ast.Module)

    global_name_dict: typing.Dict[str, ast.AST] = OrderedDict()

    def walkFn_global(t: ast.AST) -> typing.Optional[typing.Callable]:
        nonlocal global_name_dict

        if isinstance(t, (ast.SchemaStmt, ast.RuleStmt)):
            node = t
            if kcl_info.isprivate_field(node.name) or node.name not in global_name_dict:
                global_name_dict[node.name] = node
            else:
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.UniqueKeyError_TYPE,
                    file_msgs=[
                        kcl_error.ErrFileMsg(
                            filename=lark_parser.filename,
                            line_no=node.line,
                            col_no=node.column,
                        )
                    ],
                    arg_msg=kcl_error.UNIQUE_KEY_MSG.format(node.name),
                )
            return None

        if isinstance(t, (ast.SchemaExpr, ast.LambdaExpr)):
            return None

        if isinstance(t, ast.ImportStmt):
            return None

        if isinstance(t, (ast.ListComp, ast.DictComp)):
            return None

        if isinstance(t, ast.AssignStmt):
            node = typing.cast(ast.AssignStmt, t)
            for expr in node.targets:
                if not isinstance(expr, ast.Identifier) or isinstance(
                    node.value, ast.LambdaExpr
                ):
                    continue

                ident = typing.cast(ast.Identifier, expr)
                is_config = isinstance(node.value, ast.SchemaExpr)
                if (
                    kcl_info.isprivate_field(ident.names[0])
                    or (ident.names[0] not in global_name_dict)
                    or is_config
                ):
                    global_name_dict[ident.names[0]] = node
                else:
                    kcl_error.report_exception(
                        err_type=kcl_error.ErrType.ImmutableCompileError_TYPE,
                        file_msgs=[
                            kcl_error.ErrFileMsg(
                                filename=lark_parser.filename,
                                line_no=ident.line,
                                col_no=ident.column,
                                end_col_no=ident.end_column,
                            )
                        ],
                    )

        # continue walk
        return walkFn_global

    # walk tree
    ast.WalkTree(m, walkFn_global)

    # dict to list
    return list(global_name_dict.keys())


def _get_schema_local_names(
    schema: typing.Union[ast.SchemaStmt, ast.RuleStmt]
) -> typing.List[str]:
    assert schema
    assert isinstance(schema, (ast.SchemaStmt, ast.RuleStmt))

    local_name_dict: typing.Dict[str, ast.AST] = OrderedDict()

    # walk args
    if schema.args:
        for x in schema.args.args:
            assert len(x.names) == 1, f"schema.args={schema.args}"
            local_name_dict[x.names[0]] = x

    def walkFn_schema_local(t: ast.AST) -> typing.Optional[typing.Callable]:
        if isinstance(t, ast.SchemaAttr):
            node = typing.cast(ast.SchemaAttr, t)
            local_name_dict[node.name] = node
            return None

        if isinstance(t, ast.AssignStmt):
            node = typing.cast(ast.AssignStmt, t)

            # a = b = c.d = value
            for expr in node.targets:
                if not isinstance(expr, ast.Identifier):
                    continue

                ident = typing.cast(ast.Identifier, expr)

                # skip: c.d = value
                if len(ident.names) == 1:
                    local_name_dict[ident.names[0]] = ident

        # continue walk
        return walkFn_schema_local

    # walk tree
    ast.WalkTree(schema, walkFn_schema_local)

    return list(local_name_dict.keys())


def fix_set_parent_info(m: ast.Module):
    """
    set parent info on ast
    :param m: target module ast
    """

    def _walk(t: ast.AST):
        def _set_parent(
            parent: ast.AST, inner: typing.Union[typing.List, typing.Dict, ast.AST]
        ):
            if isinstance(inner, list):
                [_set_parent(parent, item) for item in inner]
                return
            if isinstance(inner, dict):
                [_set_parent(parent, v) for _, v in inner]
                return
            if isinstance(inner, ast.AST):
                inner.parent = parent
                _walk(inner)

        for _, value in ast.iter_fields(t):
            _set_parent(t, value)

    assert m and isinstance(m, ast.Module)
    return _walk(m)


def fix_qualified_identifier(
    m: ast.Module, *, import_names: typing.Optional[typing.Dict[str, str]] = None
):
    """
    import path.to.pkg as pkgname

    x = pkgname.Name
    """
    # 0. init import names
    if import_names is None or not isinstance(import_names, dict):
        import_names = {}
    for import_spec in m.GetImportList():
        import_names[import_spec.name] = import_spec.path

    # 1. init global names
    _global_names = _get_global_names(m)
    for name in _global_names:
        if name not in m.global_names:
            m.global_names.append(name)

    # 2. init schema local name
    _schema_local_names: typing.Dict[str, typing.List[str]] = {}
    for schema in m.GetSchemaAndRuleList():
        _schema_local_names[schema.name] = _get_schema_local_names(schema)
        if schema.name not in m.local_names:
            m.local_names = _schema_local_names

    current_schema_name = ""
    generator_local_vars = []

    def walkFn_fix_global_ident(t: ast.AST) -> typing.Optional[typing.Callable]:
        if isinstance(t, (ast.DictComp, ast.ListComp)):
            for gen in t.generators or []:
                for ident in gen.targets:
                    generator_local_vars.append(ident.get_first_name())
                ast.WalkTree(gen, walkFn_fix_global_ident)
            if isinstance(t, ast.ListComp):
                ast.WalkTree(t.elt, walkFn_fix_global_ident)
            if isinstance(t, ast.DictComp):
                ast.WalkTree(t.key, walkFn_fix_global_ident)
                ast.WalkTree(t.value, walkFn_fix_global_ident)
            generator_local_vars.clear()
            return None
        elif isinstance(t, ast.QuantExpr):
            for ident in t.variables:
                generator_local_vars.append(ident.get_first_name())
            ast.WalkTree(t.target, walkFn_fix_global_ident)
            ast.WalkTree(t.test, walkFn_fix_global_ident)
            ast.WalkTree(t.if_cond, walkFn_fix_global_ident)
            generator_local_vars.clear()
        if not isinstance(t, ast.Identifier):
            return walkFn_fix_global_ident

        ident = typing.cast(ast.Identifier, t)
        if len(ident.names) < 2:
            return None

        # skip global name and generator local variables in list/dict comp and quant expression
        if ident.names[0] in _global_names or ident.names[0] in generator_local_vars:
            return None

        # fix qualified identifier
        if ident.names[0] in import_names:
            ident.pkgpath = import_names[ident.names[0]]

        return None

    def walkFn_fix_schema_ident(t: ast.AST) -> typing.Optional[typing.Callable]:
        nonlocal current_schema_name
        assert current_schema_name, f"current_schema_name={current_schema_name}"

        if not isinstance(t, ast.Identifier):
            return walkFn_fix_global_ident

        ident = typing.cast(ast.Identifier, t)
        if len(ident.names) < 2:
            return None

        # skip local name
        _local_names = _schema_local_names[current_schema_name]
        if ident.names[0] in _local_names:
            return None

        # skip global name
        if ident.names[0] in _global_names:
            return None

        # fix qualified identifier
        if ident.names[0] in import_names:
            ident.pkgpath = import_names[ident.names[0]]

        return None

    # -----------------------------------------------------

    # 3. fix all ident
    for stmt in m.body or []:
        if isinstance(stmt, (ast.SchemaStmt, ast.RuleStmt)):
            node = stmt
            current_schema_name = node.name
            ast.WalkTree(node, walkFn_fix_schema_ident)
            current_schema_name = ""
            continue

        ast.WalkTree(stmt, walkFn_fix_global_ident)

    # OK
    return


def fix_and_get_module_import_list(
    root: str, m: ast.Module, is_fix: bool = True, reversed: bool = False
) -> typing.List[ast.ImportStmt]:
    assert m
    assert isinstance(m, ast.Module)
    assert m.pkg

    import_spec_list: typing.List[ast.ImportStmt] = []
    pkgpath_table = {}

    for stmt in m.body or []:
        if not isinstance(stmt, ast.ImportStmt):
            continue

        if is_fix:
            assert stmt.path
            assert stmt.pkg_name

            stmt.rawpath = stmt.path

            stmt.path = vfs.FixImportPath(root, m.filename, stmt.path)
            stmt.name = stmt.pkg_name
        if reversed:
            pkgpath_table[stmt.path] = stmt.name
        else:
            pkgpath_table[stmt.name] = stmt.path

        import_spec = copy.deepcopy(stmt)
        import_spec_list.append(import_spec)

    if not is_fix:
        return import_spec_list

    # fix types name
    # asname.Name => @abs.pkg.Name
    # [asname.Name] => [@abs.pkg.Name]
    # {str:asname.Name} => {str:@abs.pkg.Name}
    # {str:[asname.Name]} => {str:[@abs.pkg.Name]}
    # asname1.Name1 | asname2.Name2 => @abs.pkg1.Name1 | @abs.pkg2.Name2
    for stmt in m.body or []:
        if isinstance(stmt, ast.AssignStmt):
            assign_stmt = typing.cast(ast.AssignStmt, stmt)
            if assign_stmt.type_annotation:
                if reversed:
                    assign_stmt.type_annotation = re.sub(
                        PKGPATH_DOT_REGEX,
                        lambda x: f"{pkgpath_table[x.group()[1:-1]]}."
                        if x.group()[1:-1] in pkgpath_table
                        else x.group(),
                        assign_stmt.type_annotation,
                    )
                else:
                    assign_stmt.type_annotation = re.sub(
                        PKGPATH_IDENTIFIER_DOT_REGEX,
                        lambda x: f"@{pkgpath_table[x.group()[:-1]]}."
                        if x.group()[:-1] in pkgpath_table
                        else x.group(),
                        assign_stmt.type_annotation,
                    )
        elif isinstance(stmt, ast.SchemaStmt):
            schema_stmt = typing.cast(ast.SchemaStmt, stmt)
            # Fix schema arguments type
            if schema_stmt.args and schema_stmt.args.type_annotation_list:
                for i, _type in enumerate(schema_stmt.args.type_annotation_list):
                    # if the `_type` is None, the schema argument has no any type annotation
                    if not _type:
                        continue
                    if reversed:
                        schema_stmt.args.type_annotation_list[i] = re.sub(
                            PKGPATH_DOT_REGEX,
                            lambda x: f"{pkgpath_table[x.group()[1:-1]]}."
                            if x.group()[1:-1] in pkgpath_table
                            else x.group(),
                            _type,
                        )
                    else:
                        schema_stmt.args.type_annotation_list[i] = re.sub(
                            PKGPATH_IDENTIFIER_DOT_REGEX,
                            lambda x: f"@{pkgpath_table[x.group()[:-1]]}."
                            if x.group()[:-1] in pkgpath_table
                            else x.group(),
                            _type,
                        )
            # Fix schame attr type
            for attr in schema_stmt.body or []:
                if not isinstance(attr, ast.SchemaAttr):
                    continue
                schema_attr = typing.cast(ast.SchemaAttr, attr)
                _type = schema_attr.type_str
                if reversed:
                    schema_attr.type_str = re.sub(
                        PKGPATH_DOT_REGEX,
                        lambda x: f"{pkgpath_table[x.group()[1:-1]]}."
                        if x.group()[1:-1] in pkgpath_table
                        else x.group(),
                        _type,
                    )
                else:
                    schema_attr.type_str = re.sub(
                        PKGPATH_IDENTIFIER_DOT_REGEX,
                        lambda x: f"@{pkgpath_table[x.group()[:-1]]}."
                        if x.group()[:-1] in pkgpath_table
                        else x.group(),
                        _type,
                    )
        elif isinstance(stmt, ast.RuleStmt):
            rule_stmt = typing.cast(ast.RuleStmt, stmt)
            if rule_stmt.args and rule_stmt.args.type_annotation_list:
                for i, _type in enumerate(rule_stmt.args.type_annotation_list):
                    # if the `_type` is None, the rule argument has no any type annotation
                    if not _type:
                        continue
                    if reversed:
                        rule_stmt.args.type_annotation_list[i] = re.sub(
                            PKGPATH_DOT_REGEX,
                            lambda x: f"{pkgpath_table[x.group()[1:-1]]}."
                            if x.group()[1:-1] in pkgpath_table
                            else x.group(),
                            _type,
                        )
                    else:
                        rule_stmt.args.type_annotation_list[i] = re.sub(
                            PKGPATH_IDENTIFIER_DOT_REGEX,
                            lambda x: f"@{pkgpath_table[x.group()[:-1]]}."
                            if x.group()[:-1] in pkgpath_table
                            else x.group(),
                            _type,
                        )
        elif isinstance(stmt, ast.TypeAliasStmt):
            # Fix rule arguments type
            type_alias_stmt = typing.cast(ast.TypeAliasStmt, stmt)
            if type_alias_stmt.type_value.plain_type_str:
                if reversed:
                    type_alias_stmt.type_value.plain_type_str = re.sub(
                        PKGPATH_DOT_REGEX,
                        lambda x: f"{pkgpath_table[x.group()[1:-1]]}."
                        if x.group()[1:-1] in pkgpath_table
                        else x.group(),
                        type_alias_stmt.type_value.plain_type_str,
                    )
                else:
                    type_alias_stmt.type_value.plain_type_str = re.sub(
                        PKGPATH_IDENTIFIER_DOT_REGEX,
                        lambda x: f"@{pkgpath_table[x.group()[:-1]]}."
                        if x.group()[:-1] in pkgpath_table
                        else x.group(),
                        type_alias_stmt.type_value.plain_type_str,
                    )

    class TypeNameTransformer(ast.TreeTransformer):
        def walk_LambdaExpr(self, node: ast.LambdaExpr):
            if node.args and node.args.type_annotation_list:
                for i, _type in enumerate(node.args.type_annotation_list):
                    # if the `_type` is None, the schema argument has no any type annotation
                    if not _type:
                        continue
                    if reversed:
                        node.args.type_annotation_list[i] = re.sub(
                            PKGPATH_DOT_REGEX,
                            lambda x: f"{pkgpath_table[x.group()[1:-1]]}."
                            if x.group()[1:-1] in pkgpath_table
                            else x.group(),
                            _type,
                        )
                    else:
                        node.args.type_annotation_list[i] = re.sub(
                            PKGPATH_IDENTIFIER_DOT_REGEX,
                            lambda x: f"@{pkgpath_table[x.group()[:-1]]}."
                            if x.group()[:-1] in pkgpath_table
                            else x.group(),
                            _type,
                        )
            if node.return_type_str:
                if reversed:
                    node.return_type_str = re.sub(
                        PKGPATH_DOT_REGEX,
                        lambda x: f"{pkgpath_table[x.group()[1:-1]]}."
                        if x.group()[1:-1] in pkgpath_table
                        else x.group(),
                        node.return_type_str,
                    )
                else:
                    node.return_type_str = re.sub(
                        PKGPATH_IDENTIFIER_DOT_REGEX,
                        lambda x: f"@{pkgpath_table[x.group()[:-1]]}."
                        if x.group()[:-1] in pkgpath_table
                        else x.group(),
                        node.return_type_str,
                    )
            return node

    TypeNameTransformer().walk(m)

    return import_spec_list


def fix_test_schema_auto_relaxed(m: ast.Module):
    if not m.filename.endswith("_test.k"):
        return

    for stmt in m.body or []:
        if not isinstance(stmt, ast.SchemaStmt):
            continue

        schema = typing.cast(ast.SchemaStmt, stmt)
        if schema.name.startswith("Test"):
            for x in schema.body or []:
                if not isinstance(x, ast.SchemaAttr):
                    continue
                attr = typing.cast(ast.SchemaAttr, x)
                attr.type_str = ""
                attr.is_optional = True

    return
