import os
from io import StringIO
from typing import Union, Optional
import pathlib

import kclvm.kcl.ast.ast as ast
import kclvm.tools.printer.printer as printer
import kclvm.kcl.info as kcl_info


def get_source_code(
    file: Optional[Union[str, pathlib.PosixPath]], line: int, code: Optional[str] = None
) -> str:
    if code:
        lines = code.split("\n")
        assert line <= len(lines)
        source_line = lines[line - 1]
    else:
        _file = str(file)
        assert is_kcl_file(_file)
        assert line > 0
        with open(_file, "r", encoding="utf8") as source_file:
            lines = source_file.read().split("\n")
            assert line <= len(lines)
            source_line = lines[line - 1]
    return source_line


def is_kcl_file(file: str) -> bool:
    assert isinstance(file, str)
    return os.path.isfile(file) and file.endswith(kcl_info.KCL_FILE_SUFFIX)


def get_code_from_module(module: ast.Module) -> str:
    assert isinstance(module, ast.Module)
    with StringIO() as IO:
        printer.PrintAST(module, IO)
        code = IO.getvalue()
    return code
