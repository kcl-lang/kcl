from typing import List, cast, Optional
from pygls.lsp.types.basic_structures import Location

import kclvm.kcl.ast as ast
import kclvm.kcl.types.scope as scope
import kclvm.tools.langserver.common as common
from kclvm.api.object.object import KCLTypeKind, KCLModuleTypeObject
from kclvm.api.object.schema import KCLSchemaTypeObject, KCLSchemaDefTypeObject


def definition(
    pos: ast.Position, code: str = None
) -> (Optional[ast.AST], Optional[scope.ScopeObject]):
    prog: ast.Program
    prog, leaf_node = common.pos_to_node(pos, code)
    if not leaf_node:
        # no name node at the position
        return None, None
    parent: ast.AST = leaf_node.parent
    if isinstance(leaf_node, ast.Name):
        if (
            parent.type == "Identifier"
            and parent.parent.type == "ConfigEntry"
            and parent is parent.parent.key
        ):
            identifier: ast.Identifier = cast(ast.Identifier, parent)
            _, prog_scope = common.file_or_prog_to_scope(prog)
            schema_expr: ast.SchemaExpr = leaf_node.find_nearest_parent_by_type(
                ast.SchemaExpr
            )
            if schema_expr:
                schema_name: ast.Identifier = schema_expr.name
                schema_scope_obj = find_declaration(
                    schema_name, schema_name.name_nodes[-1], prog_scope
                )
                top_attr = find_inner_name(
                    schema_scope_obj, identifier.name_nodes[0], prog_scope
                )
                result_obj = find_declaration_by_scope_obj(
                    identifier=identifier,
                    name_node=leaf_node,
                    top_name_obj=top_attr,
                    prog_scope=prog_scope,
                )
                return leaf_node, result_obj
        if parent.type == "Identifier" and (
            parent.parent.type != "ConfigEntry" or parent is parent.parent.value
        ):
            identifier: ast.Identifier = cast(ast.Identifier, parent)
            _, prog_scope = common.file_or_prog_to_scope(prog)
            declaration = find_declaration(identifier, leaf_node, prog_scope)
            return leaf_node, declaration
    return leaf_node, None


def go_to_def(pos: ast.Position, code: str = None) -> List[Location]:
    prog: ast.Program
    prog, leaf_node = common.pos_to_node(pos, code)
    if not leaf_node:
        # no name node at the position
        return []
    parent: ast.AST = leaf_node.parent
    if isinstance(leaf_node, ast.Name):
        if parent.type == "ImportStmt":
            import_stmt: ast.ImportStmt = cast(ast.ImportStmt, parent)
            if leaf_node in import_stmt.path_nodes:
                index = import_stmt.path_nodes.index(leaf_node)
                if index == len(import_stmt.path_nodes) - 1:
                    # this might be a module name, return the target module file path
                    loc = common.pkgpath_to_location(
                        root=prog.root, pkgpath=import_stmt.path
                    )
                    return [loc] if loc else []
            return [common.node_to_location(leaf_node)]
        if (
            parent.type == "Identifier"
            and parent.parent.type == "ConfigEntry"
            and parent is parent.parent.key
        ):
            identifier: ast.Identifier = cast(ast.Identifier, parent)
            _, prog_scope = common.file_or_prog_to_scope(prog)
            schema_expr: ast.SchemaExpr = leaf_node.find_nearest_parent_by_type(
                ast.SchemaExpr
            )
            if schema_expr:
                schema_name: ast.Identifier = schema_expr.name
                schema_scope_obj = find_declaration(
                    schema_name, schema_name.name_nodes[-1], prog_scope
                )
                top_attr = find_inner_name(
                    schema_scope_obj, identifier.name_nodes[0], prog_scope
                )
                result_obj = find_declaration_by_scope_obj(
                    identifier=identifier,
                    name_node=leaf_node,
                    top_name_obj=top_attr,
                    prog_scope=prog_scope,
                )
                loc = common.scope_obj_to_location(result_obj)
                return [loc] if loc else []
        if parent.type == "Identifier" and (
            parent.parent.type != "ConfigEntry" or parent is parent.parent.value
        ):
            identifier: ast.Identifier = cast(ast.Identifier, parent)
            _, prog_scope = common.file_or_prog_to_scope(prog)
            declaration = find_declaration(identifier, leaf_node, prog_scope)
            loc = common.scope_obj_to_location(declaration)
            return [loc] if loc else []
    return [common.node_to_location(leaf_node)]


def find_declaration(
    identifier: ast.Identifier, name_node: ast.Name, prog_scope: scope.ProgramScope
) -> Optional[scope.ScopeObject]:
    if not identifier or not name_node or not prog_scope:
        return None
    top_name = identifier.name_nodes[0]
    top_name_obj = find_declaration_obj_by_pos_and_name(
        top_name.pos, top_name.value, prog_scope
    )
    return find_declaration_by_scope_obj(
        identifier, name_node, top_name_obj, prog_scope
    )


def find_declaration_by_scope_obj(
    identifier: ast.Identifier,
    name_node: ast.Name,
    top_name_obj: scope.ScopeObject,
    prog_scope: scope.ProgramScope,
) -> Optional[scope.ScopeObject]:
    if not identifier or not name_node or not top_name_obj or not prog_scope:
        return None
    index = identifier.name_nodes.index(name_node)
    i = 0
    obj = top_name_obj
    while i < index:
        i = i + 1
        obj = find_inner_name(obj, identifier.name_nodes[i], prog_scope)
        if not obj:
            return None
    return obj


def find_declaration_obj_by_pos_and_name(
    pos: ast.Position, name: str, prog_scope: scope.ProgramScope
) -> Optional[scope.ScopeObject]:
    if not pos or not pos.is_valid() or not name or not prog_scope:
        return None
    inner_most = prog_scope.main_scope.inner_most(pos)
    if not inner_most or not inner_most.elems:
        return None
    scope_obj = inner_most.elems.get(name)
    if scope_obj is not None:
        return scope_obj
    # 1. search through the parent schema scope tree
    parent_scope = inner_most.get_parent_schema_scope(prog_scope)
    while parent_scope is not None:
        scope_obj = parent_scope.elems.get(name)
        if scope_obj is not None:
            return scope_obj
        parent_scope = parent_scope.get_parent_schema_scope(prog_scope)
    # 2. search through the enclosing scope tree
    while inner_most is not None:
        scope_obj = inner_most.elems.get(name)
        if scope_obj is not None:
            return scope_obj
        inner_most = inner_most.get_enclosing_scope()
    return None


def find_inner_name(
    out_name_obj: scope.ScopeObject,
    inner_name: ast.Name,
    prog_scope: scope.ProgramScope,
) -> Optional[scope.ScopeObject]:
    if not out_name_obj or not inner_name or not prog_scope:
        return None
    if out_name_obj.type.type_kind() == KCLTypeKind.SchemaKind:
        return find_attr_by_name(inner_name.value, out_name_obj.type, prog_scope)
    if out_name_obj.type.type_kind() == KCLTypeKind.ModuleKind:
        out_type = cast(KCLModuleTypeObject, out_name_obj.type)
        if out_type.is_user_module:
            pkg_scope = prog_scope.scope_map.get(out_type.pkgpath)
            return pkg_scope.elems.get(inner_name.value) if pkg_scope else None
    if out_name_obj.type.type_kind() == KCLTypeKind.SchemaDefKind:
        out_type = cast(KCLSchemaDefTypeObject, out_name_obj.type)
        return find_attr_by_name(inner_name.value, out_type.schema_type, prog_scope)


def find_attr_by_name(
    attr_name: str, schema_type: KCLSchemaTypeObject, prog_scope: scope.ProgramScope
) -> Optional[scope.ScopeObject]:
    while schema_type:
        if attr_name in schema_type.attr_list:
            # todo: support jump to schema index signature
            pkg_scope = prog_scope.scope_map.get(schema_type.pkgpath)
            schema_scope = pkg_scope.search_child_scope_by_name(schema_type.name)
            return schema_scope.elems.get(attr_name) if schema_scope else None
        schema_type = schema_type.base
    return None
