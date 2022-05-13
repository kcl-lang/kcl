from typing import List

import kclvm.kcl.ast.ast as ast
import kclvm.tools.lint.message.message as message
from kclvm.tools.lint.checkers.base_checker import BaseChecker

MSGS = {
    "E0501": (
        "line too long (%d > %d characters).",
        "Line too long.",
        "line too long ('{0}' > '{1}' characters).",
    )
}


class MiscChecker(BaseChecker):
    def __init__(self, linter) -> None:
        super().__init__(linter)
        self.name = "MiscCheck"
        self.code = None
        self.MSGS = MSGS
        self.module = None
        self.prog = None
        self.work_dir = None
        self.root: str = None

    def reset(self) -> None:
        self.msgs.clear()
        self.code = None
        self.module = None
        self.root = None

    def get_module(self, prog: ast.Program, code: str) -> None:
        assert isinstance(prog, ast.Program)
        assert code is not None
        self.prog = prog
        self.module = prog.pkgs["__main__"][0]
        self.code = code
        self.root: str = prog.root

    def check(self, prog: ast.Program, code: str) -> None:
        assert isinstance(prog, ast.Program)
        assert code is not None
        self.reset()
        self.get_module(prog, code)
        code_lines = self.code.split("\n")
        self.check_line_too_long(code_lines)

    # todo: check in ast instead of code or file
    def check_line_too_long(self, code_lines: List[str]) -> None:
        for i, v in enumerate(code_lines):
            if len(v) > self.options.max_line_length:
                self.msgs.append(
                    message.Message(
                        "E0501",
                        self.module.filename,
                        MSGS["E0501"][0]
                        % (
                            len(v),
                            self.options.max_line_length,
                        ),
                        v.strip(),
                        (i + 1, 1),
                        [len(v), self.options.max_line_length],
                    )
                )
