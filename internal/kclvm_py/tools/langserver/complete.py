import pygls.lsp.types.language_features.completion as completion
import typing
import kclvm.kcl.ast as ast
from kclvm.tools.langserver.common import pos_to_scope, scope_obj_to_completion_item


def complete(
    pos: ast.Position, name: str, code: str = None
) -> typing.List[completion.CompletionItem]:
    _, inner_most, _ = pos_to_scope(pos, code)
    _scope = inner_most
    completion_list: completion.CompletionList = completion.CompletionList(
        isIncomplete=False
    )
    while _scope is not None:
        for key, obj in _scope.elems.items():
            if key.startswith(name):
                completion_list.add_item(scope_obj_to_completion_item(obj))
        _scope = _scope.parent
    return completion_list.items
