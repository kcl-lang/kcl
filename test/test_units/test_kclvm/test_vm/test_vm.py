# Copyright 2021 The KCL Authors. All rights reserved.

import pathlib
import unittest

import kclvm.kcl.error as kcl_error
import kclvm.vm as vm
import kclvm.api.object as kcl_object
import kclvm.compiler.parser as parser
import kclvm.compiler.build.compiler as compiler

from kclvm.api.object import KCLCompiledFunctionObject, Parameter


class VMTest(unittest.TestCase):
    def get_vm(self, filepath: str) -> vm.VirtualMachine:
        program = compiler.CompileProgram(parser.LoadProgram(filepath))
        return vm.VirtualMachine(program)

    def test_vm_run(self):
        filepath = str(
            pathlib.Path(__file__).parent.joinpath("test_data").joinpath("main.k")
        )
        machine = self.get_vm(filepath)
        result = machine.Run()
        self.assertEqual(result.filename, filepath)
        self.assertEqual(list(result.m.keys()), ["person", "@pkg"])
        self.assertEqual(list(machine.state.modules.keys()), ["pkg"])
        self.assertEqual(list(machine.all_schema_types.keys()), ["pkg.Person"])
        self.assertEqual(machine.last_popped_obj(), kcl_object.NONE_INSTANCE)

    def test_vm_invalid_run(self):
        filepaths = [
            str(
                pathlib.Path(__file__)
                .parent.joinpath("invalid_test_data")
                .joinpath("unification.k")
            ),
            str(
                pathlib.Path(__file__)
                .parent.joinpath("invalid_test_data")
                .joinpath("recursive.k")
            ),
        ]
        for filepath in filepaths:
            with self.assertRaises(kcl_error.KCLException):
                self.get_vm(filepath).Run()

    def test_default_not_full(self):
        app = kcl_object.KCLProgram()
        app.pkgs = {"testpkg": kcl_object.KCLBytecode()}
        app.main = "testpkg"
        test_vm = vm.VirtualMachine(app=app)

        test_f = vm.Frame()
        test_f.locals = {}
        test_f.globals = {}
        test_vm.ctx = test_f

        test_func = KCLCompiledFunctionObject(name="test_function")
        test_func.params = [
            Parameter(name="test_name", value=kcl_object.to_kcl_obj(10))
        ]
        test_vm.push_frame_using_callable(
            pkgpath="test_pkg", func=test_func, args=[], kwargs=[]
        )
        self.assertEqual(test_vm.ctx.locals["test_name"], kcl_object.to_kcl_obj(10))


if __name__ == "__main__":
    unittest.main(verbosity=2)
