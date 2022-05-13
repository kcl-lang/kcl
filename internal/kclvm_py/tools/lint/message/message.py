import pathlib
from typing import Tuple, Union, Optional, List, Any


class Message:
    """This class represent a message to be issued by the reporters"""

    def __init__(
        self,
        msg_id: str,
        file: Optional[Union[str, pathlib.PosixPath]],
        msg: str,
        source_code: str,
        pos: Tuple[int, int],
        arguments: List[Any],
    ):
        self.msg_id = msg_id
        self.level = msg_id[0]
        self.file = str(file) if file else None
        self.msg = msg
        self.source_code = source_code
        self.pos = pos
        self.arguments = [str(arg) for arg in arguments]

    def __str__(self):
        return (
            f"{self.file}:{self.pos[0]}:{self.pos[1]}: {self.msg_id} {self.msg}\n"
            + self.source_code
            + "\n"
            + (self.pos[1] - 1) * " "
            + "^"
        )

    def __eq__(self, other):
        return (
            self.msg_id,
            self.file,
            self.msg,
            self.source_code,
            self.pos,
            self.arguments,
        ) == (
            other.msg_id,
            other.file,
            other.msg,
            other.source_code,
            other.pos,
            other.arguments,
        )
