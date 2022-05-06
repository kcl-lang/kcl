import sys
from typing import List

from kclvm.tools.lint.reporters.base_reporter import BaseReporter
from kclvm.tools.lint.message.message import Message

LINT_THEME: dict = {
    "ID": "\033[0;92m{}\033[0m",  # green
    "ERROR": "\033[0;91m{}\033[0m",  # red
    "WARNING": "\033[0;93m{}\033[0m",  # yellow
    "FILE_NAME": "\033[0;94m{}\033[0m",  # blue
    "LINE_COLUMN": "\033[0;96m{}\033[0m",  # cyan
    "MARK": "\033[0;31m{}\033[0m",  # red
    "NUMBER": "\033[0;31m{}\033[0m",  # red
}


def color(content: str, content_type: str):
    return LINT_THEME[content_type].format(content)


def msg_with_color(msg: Message):
    return (
        color(msg.file, "FILE_NAME")
        + ":"
        + color(msg.pos[0], "LINE_COLUMN")
        + ":"
        + color(msg.pos[1], "LINE_COLUMN")
        + ": "
        + color(msg.msg_id, "ID")
        + " "
        + msg.msg
        + "\n"
        + msg.source_code
        + "\n"
        + (msg.pos[1] - 1) * " "
        + color("^", "MARK")
    )


class STDOUTReporter(BaseReporter):
    def __init__(self, linter, output=None, encoding=None):
        self.name = "stdout_reporter"
        super().__init__(linter, output, encoding)

    def print_msg(self, msgs: List[Message], file=sys.stdout):
        """
        Print msgs with color.Because CI cannot parse color information, e.g. [0;31m{,
        it is not enabled temporarily

        for msg in msgs:
            print((msg_with_color(msg) if file.isatty() else str(msg)) + "\n")
        print(
            "Check total "
            + (
                color(len(self.linter.file_list), "NUMBER")
                if file.isatty()
                else str(len(self.linter.file_list))
            )
            + " files:"
        )
        for k, v in self.linter.msgs_map.items():
            print(("{:<19}".format(color(v, "NUMBER"))
                   + color(k, "ID")
                   + ": "
                   + self.linter.MSGS[k][1]
                   ) if file.isatty() else ("{:<8}{}: {}".format(v, k, self.linter.MSGS[k][1])))
        print("KCL Lint: "
              + (color(len(self.linter.msgs), "NUMBER") if file.isatty() else str(len(self.linter.msgs)))
              + " problems")
        """
        for msg in msgs:
            print(msg)
            print()
        print(f"Check total {len(self.linter.file_list)} files:")
        for k, v in self.linter.msgs_map.items():
            print("{:<8}{}: {}".format(v, k, self.linter.MSGS[k][1]))
        print(f"KCL Lint: {len(self.linter.msgs)} problems")
