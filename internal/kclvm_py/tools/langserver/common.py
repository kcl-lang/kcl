import typing
import pathlib

import kclvm.compiler.parser.parser as parser
import kclvm.kcl.ast as ast
import kclvm.kcl.types.scope as scope
import kclvm.kcl.error as kcl_error

from kclvm.kcl.types import ResolveProgram, ProgramScope, CheckConfig
from pygls.lsp.types.basic_structures import Location, Range, Position
from pygls.lsp.types.language_features.completion import (
    CompletionItemKind,
    CompletionItem,
)
from kclvm.api.object import KCLTypeKind
from kclvm.compiler.vfs.vfs import get_pkg_realpath_from_pkgpath as get_realpath

INNER_TYPE_2_COMPLETION_ITEM_KIND = {
    # any type
    KCLTypeKind.AnyKind: CompletionItemKind.Text,
    # base types
    KCLTypeKind.BoolKind: CompletionItemKind.Value,
    KCLTypeKind.IntKind: CompletionItemKind.Value,
    KCLTypeKind.FloatKind: CompletionItemKind.Value,
    KCLTypeKind.StrKind: CompletionItemKind.Value,
    # constants
    KCLTypeKind.NoneKind: CompletionItemKind.Constant,
    KCLTypeKind.BoolLitKind: CompletionItemKind.Constant,
    KCLTypeKind.FloatLitKind: CompletionItemKind.Constant,
    KCLTypeKind.IntLitKind: CompletionItemKind.Constant,
    KCLTypeKind.StrLitKind: CompletionItemKind.Constant,
    # schema
    KCLTypeKind.SchemaKind: CompletionItemKind.Struct,
    KCLTypeKind.SchemaDefKind: CompletionItemKind.Struct,
    # built in function
    KCLTypeKind.FuncKind: CompletionItemKind.Function,
}


def pos_to_node(
    pos: ast.Position, code: str = None
) -> (typing.Optional[ast.Program], typing.Optional[ast.AST]):
    if not pos or not pos.is_valid():
        return None, None
    prog = file_to_prog(pos.filename, code)
    if not prog:
        return None, None
    for module in prog.pkgs[ast.Program.MAIN_PKGPATH]:
        leaf_node = module.find_leaf_by_pos(pos)
        if leaf_node:
            return prog, leaf_node
    return prog, None


def pos_to_scope(
    pos: ast.Position, code: str = None
) -> (
    typing.Optional[ast.Program],
    typing.Optional[scope.Scope],
    typing.Optional[scope.Scope],
):
    if not pos or not pos.is_valid():
        return None, None, None
    prog, prog_scope = file_or_prog_to_scope(None, pos.filename, code)
    if not prog_scope or not prog_scope.main_scope:
        return None, None, None
    return prog, prog_scope.main_scope.inner_most(pos), prog_scope


def file_or_prog_to_scope(
    prog: typing.Optional[ast.Program] = None, file_path: str = None, code: str = None
) -> (typing.Optional[ast.Program], typing.Optional[ProgramScope]):
    prog = prog or file_to_prog(file_path, code)
    if not prog:
        return None, None
    no_raise = CheckConfig()
    no_raise.raise_err = False
    try:
        return prog, ResolveProgram(prog, config=no_raise)
    except Exception:
        return None, None


def file_to_prog(file_path: str, code: str = None) -> typing.Optional[ast.Program]:
    code_list = [code] if code is not None else []
    try:
        prog = parser.LoadProgram(
            file_path,
            mode=parser.ParseMode.Null,
            k_code_list=code_list,
            set_ast_parent=True,
        )
        return prog
    except kcl_error.KCLError:
        return None
    except Exception:
        return None


def file_to_ast(file_path: str, code: str = None) -> typing.Optional[ast.Module]:
    try:
        module = parser.ParseFile(file_path, code)
        return module
    except Exception:
        return None


def scope_obj_to_location(obj: scope.ScopeObject) -> typing.Optional[Location]:
    if obj and obj.check_pos_valid():
        if obj.node.type in [
            "SchemaStmt",
            "RuleStmt",
            "SchemaAttr",
            "SchemaIndexSignature",
        ]:
            return node_to_location(obj.node.name_node)
        if obj.node.type == "Identifier":
            return node_to_location(obj.node.name_nodes[0])
        if obj.node.type == "ImportStmt":
            return node_to_location(
                obj.node.as_name_node
                if obj.node.as_name_node
                else obj.node.path_nodes[-1]
            )
    return None


def node_to_location(node: ast.AST) -> typing.Optional[Location]:
    return (
        Location(
            uri=str(pathlib.Path(node.filename)),
            range=Range(
                start=kcl_pos_to_lsp_pos(node.pos),
                end=kcl_pos_to_lsp_pos(node.end_pos),
            ),
        )
        if node
        and node.pos
        and node.pos.is_valid()
        and node.end_pos
        and node.end_pos.is_valid()
        else None
    )


def kcl_pos_to_lsp_pos(pos: ast.Position) -> Position:
    return Position(line=pos.line - 1, character=pos.column - 1)


def scope_obj_to_completion_item(obj: scope.ScopeObject) -> CompletionItem:
    assert obj and obj.name and obj.type
    return CompletionItem(
        label=obj.name,
        kind=INNER_TYPE_2_COMPLETION_ITEM_KIND.get(obj.type.type_kind()),
    )


def pkgpath_to_location(root: str, pkgpath: str) -> typing.Optional[Location]:
    filepath = get_realpath_from_pkgpath(root, pkgpath)
    return file_to_location(filepath)


def file_to_location(filepath: str) -> typing.Optional[Location]:
    if not filepath or not is_kcl_file(filepath):
        return None
    return Location(uri=filepath, range=emptyRange())


def is_kcl_file(filepath: str) -> bool:
    return filepath.endswith(".k")


def emptyRange() -> Range:
    return Range(
        start=Position(line=0, character=0),
        end=Position(line=0, character=0),
    )


def get_realpath_from_pkgpath(root: str, pkgpath: str) -> str:
    return str(pathlib.Path(get_realpath(root, pkgpath)).absolute())
