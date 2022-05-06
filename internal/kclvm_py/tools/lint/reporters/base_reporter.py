from typing import List

from kclvm.tools.lint.message.message import Message


class BaseReporter:
    name: str = ""
    """base class for reporters"""

    def __init__(self, linter, output=None, encoding=None):
        self.linter = linter
        self.out = None
        self.out_encoding = None
        self.set_output(output, encoding)

    def __eq__(self, other):
        return self.name == other.name and self.linter == other.linter

    def set_output(self, output=None, encoding="utf-8"):
        """
        set output stream
        todo:
        The output property is not used yet， and replaced by open_output_stream.
        When reporter display the result, the 'open_output_stream' function is
        called to open the output stream.
        """
        self.out = output
        self.out_encoding = encoding

    def display(self):
        self.print_msg(self.linter.msgs)

    def print_msg(self, msgs: List[Message] = None):
        """Should be overridden by subclass"""
        raise NotImplementedError()
