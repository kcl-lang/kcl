# Copyright 2021 The KCL Authors. All rights reserved.

import pathlib
import unittest

import kclvm.compiler.parser as parser
import kclvm.compiler.build.compiler as compiler
import kclvm.vm.code.code as code
import kclvm.vm.code.code_factory as code_factory
import kclvm.vm as vm


TEST_PKG_PATH = "__main__"
TEST_FILE_META = ("main.k", 1, 1)
TEST_SCHEMA_NAME = "TestPerson"
TEST_CODE = """\
schema Person:
    name: str = "Alice"
    age: int = 18

person = Person {}
"""


class VMCodeTest(unittest.TestCase):
    def test_code_factory(self):
        codes = [code.Opcode.INVALID, code.Opcode.POP_TOP, TEST_FILE_META]
        opcode_factory = code_factory.OpcodeFactory.build_from_codes(
            codes, TEST_PKG_PATH
        )
        opcode_factory.pretty_print()
        opcode_factory.values = []
        opcode_factory.pretty_print()
        program = compiler.CompileProgram(
            parser.LoadProgram(TEST_FILE_META[0], k_code_list=[TEST_CODE])
        )
        opcode_factory = code_factory.SchemaBodyOpcodeFactory.build_from_codes(
            program.pkgs[TEST_PKG_PATH].instructions, TEST_PKG_PATH, TEST_SCHEMA_NAME
        )
        opcode_factory.pretty_print()
        opcode_factory.values = []
        opcode_factory.pretty_print()

    def test_code_actions(self):
        program = compiler.CompileProgram(
            parser.LoadProgram(TEST_FILE_META[0], k_code_list=[TEST_CODE])
        )
        for opcode in [
            code.Opcode.DEBUG_GLOBALS,
            code.Opcode.DEBUG_LOCALS,
            code.Opcode.DEBUG_NAMES,
            code.Opcode.DEBUG_STACK,
        ]:
            program.pkgs[TEST_PKG_PATH].instructions.extend(
                [opcode, 0, 0, 0, TEST_FILE_META]
            )
        vm.Run(program)


if __name__ == "__main__":
    unittest.main(verbosity=2)
