# Copyright 2021 The KCL Authors. All rights reserved.

from typing import Optional

import kclvm.kcl.ast as ast
import kclvm.compiler.parser.parser as parser
import kclvm.vm as vm
import kclvm.vm.planner as planner
import kclvm.compiler.build.compiler as compiler

MAIN_PKG_NAME = "__main__"


def EvalCode(filename: str, code: Optional[str] = None) -> str:
    # Parser
    module = parser.ParseFile(filename, code)
    return EvalAST(module)


def EvalAST(module: ast.Module) -> str:
    module.pkg = MAIN_PKG_NAME
    # Compiler
    bytecode = compiler.CompileProgram(
        ast.Program(
            root=MAIN_PKG_NAME, main=MAIN_PKG_NAME, pkgs={MAIN_PKG_NAME: [module]}
        )
    )
    # VM run
    result = vm.Run(bytecode)
    # YAML plan
    return planner.YAMLPlanner().plan(result.filter_by_path_selector())
