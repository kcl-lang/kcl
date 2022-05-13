# Copyright 2021 The KCL Authors. All rights reserved.

import os
import shutil
import unittest
import pathlib

import kclvm.api.object as objpkg
import kclvm.api.object.internal as obj_internal
import kclvm.kcl.ast as ast
import kclvm.kcl.types as types
import kclvm.compiler.vfs as vfs
from kclvm.kcl.error import KCLCompileException
from kclvm.kcl.error.kcl_error import IllegalArgumentSyntaxError
from kclvm.kcl.types import ProgramScope, Scope
from kclvm.compiler.parser import LoadProgram
from kclvm.compiler.build.compiler import RuntimeCode, CompileProgram, Compiler, SymbolScope
from kclvm.compiler.extension.builtin import get_builtin_func_objects
from kclvm.vm.code import Opcode, Label

code_empty_list = ["""""", """"""]
code_simple_list = [
    """
schema Person:
    name: str

a = 1
""",
    """
b = a
person = Person {
    name: "Alice"
}
""",
]
path = str(pathlib.Path(__file__).parent)


class KCLCompilerBuildTest(unittest.TestCase):
    def test_runtime_code(self):
        runtime_code = RuntimeCode(
            names=[],
            constants=[],
            codes=[],
        )
        self.assertEqual(runtime_code.type(), objpkg.KCLObjectType.RUNTIME_CODE)
        self.assertEqual(runtime_code.type_str(), "runtime_code")

    def test_compile_program_empty(self):
        compiled_program = CompileProgram(LoadProgram(path, k_code_list=code_empty_list))
        self.assertEqual(compiled_program.main, "__main__")
        self.assertEqual(len(compiled_program.pkgs[compiled_program.main].names), 0)
        self.assertEqual(
            len(compiled_program.pkgs[compiled_program.main].constants),
            len(get_builtin_func_objects()),
        )

    def test_compile_program_simple(self):
        compiled_program = CompileProgram(
            LoadProgram(path, k_code_list=code_simple_list)
        )
        self.assertEqual(compiled_program.main, "__main__")
        self.assertEqual(len(compiled_program.pkgs[compiled_program.main].names), 9)

    def test_compile_program_invalid(self):
        testdata_path = pathlib.Path(__file__).parent.joinpath("invalid_testdata")
        cases = testdata_path.glob("*.k")
        for case in cases:
            with self.assertRaises(KCLCompileException):
                CompileProgram(LoadProgram(case, work_dir=str(testdata_path)))


    def test_compile_program_invalid_nest_import(self):
        testdata_path = pathlib.Path(__file__).parent.joinpath("invalid_testdata").joinpath("nest_import")
        cases = testdata_path.glob("main.k")
        for case in cases:
            with self.assertRaises(KCLCompileException):
                CompileProgram(LoadProgram(case, work_dir=str(testdata_path)))


class KCLCompilerWalkerFunctionTest(unittest.TestCase):
    def setUp(self):
        scope = ProgramScope(scope_map={"__main__": Scope()})
        self.fake_compiler = Compiler(scope)
        self.fake_compiler.pkgpath = "__main__"
        self.fake_compiler.pkg_scope = program_scope = types.ResolveProgram(
            LoadProgram(path, k_code_list=code_simple_list)
        ).main_scope
        self.err_compiler = Compiler(scope)
        self.err_compiler.scopes = None
        super().setUp()

    def test_generic_compiler_functions(self):
        # Expr
        numberlit = ast.NumberLit(value=1)
        # Stmt
        exprstmt = ast.ExprStmt()
        exprstmt.exprs = [numberlit]
        # Module
        module = ast.Module()
        module.body = [exprstmt]
        self.fake_compiler.generic_walk(module)
        self.fake_compiler.generic_walk(exprstmt)
        self.fake_compiler.generic_walk(numberlit)
        self.fake_compiler.expr(None)
        self.fake_compiler.stmt(None)

    def test_generic_compiler_functions_invalid(self):
        with self.assertRaises(KCLCompileException):
            self.fake_compiler.generic_walk(None)
        with self.assertRaises(KCLCompileException):
            self.fake_compiler.raise_err("Raise an error")
        with self.assertRaises(KCLCompileException):
            self.fake_compiler.change_operand(0, 100, 0)
        with self.assertRaises(KCLCompileException):
            self.err_compiler.leave_scope()
        with self.assertRaises(KCLCompileException):
            self.err_compiler.add_instruction([0])
        with self.assertRaises(Exception):
            self.fake_compiler.make_func_with_content(None, None)
        with self.assertRaises(AttributeError):
            self.err_compiler.enter_scope()

    def test_emit_call_invalid(self):
        keyword = ast.Keyword()
        keyword.arg = ast.Identifier(names=["x"])
        keyword.value = ast.NumberLit(value=1)
        keywords = [keyword, keyword]
        with self.assertRaises(KCLCompileException):
            self.fake_compiler.emit_call([], keywords)

    def test_set_jmp_invalid(self):
        op, label = Opcode.NOP, Label()
        with self.assertRaises(KCLCompileException):
            self.fake_compiler.set_jmp(op, label)

    def test_op_decorator(self):
        keyword = ast.Keyword()
        keyword.arg = ast.Identifier(names=["x"])
        keyword.value = ast.NumberLit(value=1)
        args = ast.CallExpr()
        args.args = []
        args.keywords = [keyword]
        cases = [
            {
                "name": "Deprecated",
                "key": "Person",
                "args": None,
                "target": obj_internal.DecoratorTargetType.SCHEMA_TYPE,
            },
            {
                "name": "Deprecated",
                "key": "Person",
                "args": args,
                "target": obj_internal.DecoratorTargetType.SCHEMA_TYPE,
            },
        ]
        for case in cases:
            name, key, args, target = case["name"], case["key"], case["args"], case["target"]
            self.fake_compiler.op_decorator(name, key, args, target)

    def test_op_decorator_invalid(self):
        keyword = ast.Keyword()
        keyword.arg = ast.Identifier(names=["x"])
        keyword.value = ast.NumberLit(value=1)
        args = ast.CallExpr()
        args.args = []
        args.keywords = [keyword, keyword]
        cases = [
            {
                "name": "Deprecated",
                "key": "Person",
                "args": args,
                "target": obj_internal.DecoratorTargetType.SCHEMA_TYPE,
            },
            {
                "name": None,
                "key": "Person",
                "args": args,
                "target": obj_internal.DecoratorTargetType.SCHEMA_TYPE,
            },
        ]
        for case in cases:
            name, key, args, target = case["name"], case["key"], case["args"], case["target"]
            with self.assertRaises(KCLCompileException):
                self.fake_compiler.op_decorator(name, key, args, target)

    def test_store_symbol(self):
        self.fake_compiler.store_symbol("key")
        with self.assertRaises(KCLCompileException):
            self.fake_compiler.store_symbol("key")
        self.fake_compiler.store_symbol("_key", scope=SymbolScope.INTERNAL)
        self.fake_compiler.store_symbol("_key", scope=SymbolScope.INTERNAL)

    def test_load_symbol(self):
        with self.assertRaises(KCLCompileException):
            self.fake_compiler.load_symbol(None)
        with self.assertRaises(KCLCompileException):
            self.fake_compiler.load_symbol("_err_test_key")

    def test_walk_module(self):
        module = ast.Module()
        self.fake_compiler.walk_Module(module)

    def test_walk_expr_stmt(self):
        expr_stmt = ast.ExprStmt()
        expr_stmt.exprs = [ast.NumberLit(value=1)]
        self.fake_compiler.walk_ExprStmt(expr_stmt)
        self.fake_compiler._is_in_schema_stmt.append(True)
        self.fake_compiler.walk_ExprStmt(expr_stmt)
        self.fake_compiler._is_in_schema_stmt.pop()

    def test_walk_schema_stmt_invalid(self):
        schema_stmt = ast.SchemaStmt()
        schema_stmt.pkgpath = "__main__"
        schema_stmt.name = "Person"
        schema_stmt.decorators = [ast.Decorator()]
        with self.assertRaises(AttributeError):
            self.fake_compiler.walk_SchemaStmt(schema_stmt)
        # Invalid schema mixin name
        name = ast.Identifier(names=["pkg", "MixinError"])
        name.pkgpath = "@pkg"
        schema_stmt.mixins = [name]
        with self.assertRaises(KCLCompileException):
            self.fake_compiler.walk_SchemaStmt(schema_stmt)
        # Invalid schema parent name
        schema_stmt.parent_name = ast.Identifier(names=["PersonMixin"])
        with self.assertRaises(KCLCompileException):
            self.fake_compiler.walk_SchemaStmt(schema_stmt)

    def test_walk_schema_attr_invalid(self):
        schema_attr = ast.SchemaAttr()
        schema_attr.decorators = [ast.Decorator()]
        with self.assertRaises(AttributeError):
            self.fake_compiler.walk_SchemaAttr(schema_attr)

    def test_make_func_with_content_invalid_by_defaults(self):
        file_path = str(pathlib.Path(__file__).parent.joinpath("invalid_testdata/defaults_not_full_invalid/main.k"))
        try:
            CompileProgram(LoadProgram(file_path))
        except IllegalArgumentSyntaxError as err:
            self.assertEqual(err.arg_msg, "non-default argument follows default argument")

    def test_walk_index_signature_invalid(self):
        with self.assertRaises(KCLCompileException):
            t = ast.SchemaIndexSignature()
            t.key_type = "err_str"
            self.fake_compiler.walk_SchemaIndexSignature(t)

    def test_walk_unary_expr_invalid(self):
        unary_expr = ast.UnaryExpr()
        with self.assertRaises(KCLCompileException):
            self.fake_compiler.walk_UnaryExpr(unary_expr)

    def test_walk_binary_expr(self):
        bin_expr = ast.BinaryExpr()
        bin_expr.left = ast.NumberLit(value=1)
        bin_expr.right = ast.NumberLit(value=1)
        bin_expr.op = ast.BinOp.Add
        self.fake_compiler.walk_BinaryExpr(bin_expr)
        bin_expr.op = ast.CmpOp.Eq
        self.fake_compiler.walk_BinaryExpr(bin_expr)
        bin_expr.op = ast.UnaryOp.Invert
        with self.assertRaises(KCLCompileException):
            self.fake_compiler.walk_BinaryExpr(bin_expr)

    def test_walk_selector_expr(self):
        selector_expr = ast.SelectorExpr()
        key = ast.Identifier(names=["selector_key"])
        key.ctx = ast.ExprContext.STORE
        self.fake_compiler.walk_Identifier(key)
        key.ctx = ast.ExprContext.LOAD
        selector_expr.value = key
        selector_expr.attr = ast.Identifier(names=["key"])
        selector_expr.ctx = ast.ExprContext.LOAD
        self.fake_compiler.walk_SelectorExpr(selector_expr)
        selector_expr.has_question = True
        self.fake_compiler.walk_SelectorExpr(selector_expr)
        selector_expr.ctx = ast.ExprContext.AUGLOAD
        self.fake_compiler.walk_SelectorExpr(selector_expr)
        selector_expr.ctx = ast.ExprContext.AUGSTORE
        self.fake_compiler.walk_SelectorExpr(selector_expr)

    def test_walk_subscript(self):
        subscript = ast.Subscript()
        subscript.value = ast.StringLit(value="123")
        subscript.index = ast.NumberLit(value=0)
        subscript.has_question = True
        self.fake_compiler.walk_Subscript(subscript) 


class KCLCompilerProgramScopeTest(unittest.TestCase):
    def test_get_type_from_identifier(self):
        file_path = str(pathlib.Path(__file__).parent.joinpath("scope_testdata/main.k"))
        scope = types.ResolveProgram(
            LoadProgram(file_path)
        )
        compiler = Compiler(scope)
        self.assertIsInstance(compiler.get_type_from_identifier(None), objpkg.KCLAnyTypeObject)
        identifier = ast.Identifier(names=["Sub"])
        self.assertIsInstance(compiler.get_type_from_identifier(identifier), objpkg.KCLSchemaDefTypeObject)
        del compiler.pkg_scope.elems["Sub"]
        self.assertIsInstance(compiler.get_type_from_identifier(identifier), objpkg.KCLAnyTypeObject)
        identifier = ast.Identifier(names=["pkg", "Base"])
        self.assertIsInstance(compiler.get_type_from_identifier(identifier), objpkg.KCLAnyTypeObject)
        identifier.pkgpath = "@pkg"
        self.assertIsInstance(compiler.get_type_from_identifier(identifier), objpkg.KCLSchemaDefTypeObject)
        compiler.pkg_scope.elems["@pkg"].type = None
        self.assertIsInstance(compiler.get_type_from_identifier(identifier), objpkg.KCLAnyTypeObject)
        with self.assertRaises(KCLCompileException):
            compiler.get_type_from_identifier(ast.Identifier(names=["pkg", "to", "type"]))


class KCLCompilerBuildCacheTest(unittest.TestCase):
    def test_compile_cache(self):
        cache_path = str(pathlib.Path(__file__).parent.joinpath("cache_testdata/.kclvm/"))
        file_path = str(pathlib.Path(__file__).parent.joinpath("cache_testdata/main.k"))
        compiled_program = CompileProgram(LoadProgram(file_path))
        compiled_program = CompileProgram(LoadProgram(file_path))
        if os.path.exists(cache_path):
            shutil.rmtree(cache_path)

    def test_compile_expired_cache(self):
        test_data_path_name = "cache_expired_testdata"
        rename_test_data_path = "rename_cache_expired_testdata"
        root = str(pathlib.Path(__file__).parent.joinpath(test_data_path_name))
        cache_path = str(pathlib.Path(__file__).parent.joinpath(f"{test_data_path_name}/.kclvm/"))
        file_path = str(pathlib.Path(__file__).parent.joinpath(f"{test_data_path_name}/main.k"))
        rename_test_data_path = str(pathlib.Path(__file__).parent.joinpath(rename_test_data_path))
        ast_program = LoadProgram(file_path)
        compiled_program = CompileProgram(ast_program)
        cached_ast = vfs.LoadPkgCache(root, "pkg.pkg1")
        cached_bytecode = vfs.LoadBytecodeCache(root, ast_program)
        self.assertIsNotNone(cached_ast)
        self.assertIsNotNone(cached_bytecode)
        # Rename root to test cache expired
        os.rename(root, rename_test_data_path)
        cached_ast = vfs.LoadPkgCache(rename_test_data_path, "pkg.pkg1")
        cached_bytecode = vfs.LoadBytecodeCache(rename_test_data_path, ast_program)
        self.assertIsNotNone(cached_ast)
        self.assertIsNotNone(cached_bytecode)
        # Clear temp file
        os.rename(rename_test_data_path, root)
        if os.path.exists(cache_path):
            shutil.rmtree(cache_path)


if __name__ == "__main__":
    unittest.main(verbosity=2)
