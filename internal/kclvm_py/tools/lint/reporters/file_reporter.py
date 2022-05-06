import sys
from typing import List

from kclvm.tools.lint.reporters.base_reporter import BaseReporter
from kclvm.tools.lint.message.message import Message


class FileReporter(BaseReporter):
    def __init__(self, linter, output=None, encoding=None):
        self.name = "file_reporter"
        self.output_file = linter.config.output_path
        super().__init__(linter, output, encoding)

    def print_msg(self, msgs: List[Message] = None):
        assert self.output_file
        with open(self.output_file, "w") as f:
            current = sys.stdout
            sys.stdout = f
            for msg in msgs:
                print(msg)
                print()
            print("Check total {} files:".format(len(self.linter.file_list)))
            for k, v in self.linter.msgs_map.items():
                print("{:<8}{}: {}".format(v, k, self.linter.MSGS[k][1]))
            print(f"KCL Lint: {len(self.linter.msgs)} problems")
            sys.stdout = current
