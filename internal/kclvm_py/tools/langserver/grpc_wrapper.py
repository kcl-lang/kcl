import json
from typing import Union, Optional

from kclvm.internal.gpyrpc.gpyrpc_pb2 import Position
import kclvm.kcl.ast as ast
from .complete import complete
from .go_to_def import go_to_def
from .document_symbol import document_symbol
from .hover import hover


def go_to_def_wrapper(pos: Position, code: str = None) -> str:
    pos_wrapper = proto_pos_to_ast_pos(pos)
    result = go_to_def(pos=pos_wrapper, code=code)
    return json.dumps(obj=result, default=lambda x: x.__dict__)


def complete_wrapper(pos: Position, name: str, code: str = None) -> str:
    pos_wrapper = proto_pos_to_ast_pos(pos)
    result = complete(pos=pos_wrapper, name=name, code=code)
    return json.dumps(obj=result, default=lambda x: x.__dict__)


def proto_pos_to_ast_pos(pos: Position) -> ast.Position:
    return ast.Position(filename=pos.filename, line=pos.line + 1, column=pos.column + 1)


class SnakeToCamel(json.JSONEncoder):
    """Class attributes need to be converted to camel-case notation because client is expecting that."""

    def to_camel(self, s: str):
        return "".join(
            word.capitalize() if idx > 0 else word
            for idx, word in enumerate(s.split("_"))
        )

    def convert_key(
        self, instance: Union[int, float, str, bool, list, dict, tuple]
    ) -> Optional[Union[int, float, str, bool, list, dict, tuple]]:
        if instance is None:
            return None
        if isinstance(instance, (bool, int, float, str)):
            return instance
        elif isinstance(instance, (list, set, tuple)):
            return [self.convert_key(v) for v in instance]
        elif isinstance(instance, dict):
            return {self.to_camel(k): self.convert_key(v) for k, v in instance.items()}
        else:
            return self.convert_key(instance.__dict__)

    def default(self, obj):
        return self.convert_key(obj.__dict__)


def document_symbol_wrapper(file: str, code: str = None) -> str:
    result = document_symbol(file=file, code=code)
    return json.dumps(obj=result, cls=SnakeToCamel)


def hover_wrapper(pos: Position, code: str) -> str:
    pos_wrapper = proto_pos_to_ast_pos(pos)
    result = hover(pos=pos_wrapper, code=code)
    return json.dumps(obj=result, default=lambda x: x.__dict__)
