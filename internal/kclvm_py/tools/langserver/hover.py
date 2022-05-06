import typing
import pygls.lsp.types.language_features.hover as pygls_hover
from pygls.lsp.types.basic_structures import MarkupContent, MarkupKind, Range
import kclvm.kcl.ast as ast
import kclvm.api.object as objpkg
from kclvm.kcl.types.scope import ScopeObject
import kclvm.tools.langserver.common as common
from . import go_to_def


def hover(pos: ast.Position, code: str = None) -> typing.Optional[pygls_hover.Hover]:
    def_node, def_obj = go_to_def.definition(pos, code)
    if not def_node:
        return None
    if def_obj:
        return scope_obj_desc(def_node, def_obj)
    else:
        return ast_node_desc(def_node)


def ast_node_desc(node: ast.AST) -> typing.Optional[pygls_hover.Hover]:
    if isinstance(node, ast.Name):
        return pygls_hover.Hover(
            contents=MarkupContent(
                kind=MarkupKind.PlainText,
                value=node.value,
            ),
            range=Range(
                start=common.kcl_pos_to_lsp_pos(node.pos),
                end=common.kcl_pos_to_lsp_pos(node.end_pos),
            ),
        )
    return None


def scope_obj_desc(
    node: ast.AST, obj: ScopeObject
) -> typing.Optional[pygls_hover.Hover]:
    if isinstance(node, ast.AST) and isinstance(obj, ScopeObject):
        # 针对每种类型的 scope object，进行不同的显示
        if not obj.node and isinstance(obj.type, objpkg.KCLFunctionTypeObject):
            # the target scope object is a built-in function name
            msg = (
                f"(built-in) {obj.name}("
                + ", ".join([param.param_doc() for param in obj.type.params])
                + f"): {obj.type.return_type.type_str() if obj.type.return_type else 'any'}"
                + f"\n{obj.type.doc}"
                if obj.type.doc
                else ""
            )
        else:
            msg = (
                f"{obj.name}\ntype: {obj.type.type_str()}\ndefined in:{obj.node.filename}"
                if obj.node
                else obj.name
            )
        return pygls_hover.Hover(
            contents=MarkupContent(
                kind=MarkupKind.PlainText,
                value=msg,
            ),
            range=Range(
                start=common.kcl_pos_to_lsp_pos(node.pos),
                end=common.kcl_pos_to_lsp_pos(node.end_pos),
            ),
        )
    return None
