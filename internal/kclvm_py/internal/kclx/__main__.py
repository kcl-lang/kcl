# Copyright 2021 The KCL Authors. All rights reserved.

import sys
from dataclasses import dataclass

import kclvm.compiler.parser as parser
import kclvm.compiler.build.compiler as compiler

from .transformer import transform_ast_to_kclx_ast_json_str

USAGE = """\
usage: kclvm -m kclvm.internal.kclx -f=<file>
usage: kclvm -m kclvm.internal.kclx -h
"""


@dataclass
class CmdFlags:
    help: bool = False
    file: str = ""


def parse_flags(args: list) -> CmdFlags:
    m = CmdFlags()
    for s in args:
        if s == "-h" or s == "-help":
            m.help = True
            continue

        if s.startswith("-f="):
            value = s[len("-f=") :]
            m.file = value
            continue

    return m


def main():
    flags = parse_flags(sys.argv[1:])

    if flags.help:
        print(USAGE)
        sys.exit(0)

    if flags.file:
        ast_prog = parser.LoadProgram(*flags.file.split(","))
        compiler.FixAndResolveProgram(ast_prog)
        print(transform_ast_to_kclx_ast_json_str(ast_prog))


if __name__ == "__main__":
    main()
