""" The `code` module mainly contains the definition of KCL
bytecode. The bytecode factory is used for display debugging
in the `code_factory` file and the corresponding bytecode
execution function is in the `code_actions` file.

Each bytecode corresponds to an execution function. For example,
a binary addition operation corresponds to `Opcode.BINARY_ADD`.

KCL Bytecodes are mainly divided into three categories:
* Stack operation bytecodes
* KCL semantic related bytecode
* Internal bytecodes for debugging

:copyright: Copyright 2020 The KCL Authors. All rights reserved.
"""

from .code import (
    Opcode,
    Label,
    JumpAbs,
    JumpRel,
    Instruction,
    InstructionWithArg,
    EmittedInstruction,
    CompilationScope,
)
from .code_factory import SchemaBodyOpcodeFactory
from .code_actions import VM_OP_ACTIONS

__all__ = [
    "Opcode",
    "Label",
    "Instruction",
    "InstructionWithArg",
    "JumpAbs",
    "JumpRel",
    "EmittedInstruction",
    "CompilationScope",
    "SchemaBodyOpcodeFactory",
    "VM_OP_ACTIONS",
]
