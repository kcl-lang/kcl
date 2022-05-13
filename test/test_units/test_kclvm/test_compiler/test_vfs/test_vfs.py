# Copyright 2020 The KCL Authors. All rights reserved.

import os
import pathlib
import unittest
import shutil

import kclvm.api.object as obj
import kclvm.kcl.ast as ast
import kclvm.compiler.vfs.vfs as vfs
import kclvm.compiler.parser as parser
import kclvm.compiler.build.compiler as compiler


class TestVfsLoadPkgCache(unittest.TestCase):
    def test_invalid_parameters(self):
        self.assertEqual(vfs.LoadPkgCache(None, None), None)
        self.assertEqual(vfs.LoadPkgCache(None, ""), None)
        self.assertEqual(vfs.LoadPkgCache("", None), None)
        self.assertEqual(vfs.LoadPkgCache("", ""), None)

    def test_missing_cache(self):
        self.assertEqual(
            vfs.LoadPkgCache(os.path.dirname(__file__), "pkgpath.missing"), None
        )


class TestVfsSavePkgCache(unittest.TestCase):
    def test_invalid_parameters(self):
        self.assertEqual(vfs.SavePkgCache(None, None, None), None)
        self.assertEqual(vfs.SavePkgCache(None, "", ""), None)
        self.assertEqual(vfs.SavePkgCache("", None, ""), None)
        self.assertEqual(vfs.SavePkgCache("", "", None), None)
        self.assertEqual(vfs.SavePkgCache(None, None, ""), None)
        self.assertEqual(vfs.SavePkgCache("", None, None), None)
        self.assertEqual(vfs.SavePkgCache(None, "", None), None)

    def test_save(self):
        vfs.SavePkgCache(
            os.path.dirname(__file__), "pkgpath.test_save", {"key": "value"}
        )

    def tearDown(self):
        cache_root = f"{os.path.dirname(__file__)}/.kclvm"
        if os.path.exists(cache_root):
            shutil.rmtree(cache_root)


class TestVfsIsAbsPkgPath(unittest.TestCase):
    def test_is_abs_pkg_path_with_true_case(self):
        tests = ["a", "a.b.c"]
        for i in range(len(tests)):
            self.assertTrue(vfs.IsAbsPkgPath(tests[i]), msg=f"{tests[i]}")

    def test_is_abs_pkg_path_with_false_case(self):
        tests = ["", ".", ".a", "a..b", "a b", None, 1]
        for i in range(len(tests)):
            self.assertFalse(vfs.IsAbsPkgPath(tests[i]), msg=f"{tests[i]}")


class TestVfsIsRelPkgPath(unittest.TestCase):
    def test_is_abs_pkg_path_with_true_case(self):
        tests = [".", ".a", ".a.b.c", "..a.b", " .a.b.c"]
        for i in range(len(tests)):
            self.assertTrue(vfs.IsRelPkgPath(tests[i]), msg=f"{tests[i]}")

    def test_is_abs_pkg_path_with_false_case(self):
        tests = ["", "a", "a.b.c", "a b", None, 1]
        for i in range(len(tests)):
            self.assertFalse(vfs.IsRelPkgPath(tests[i]), msg=f"{tests[i]}")


class TestFixImportPath(unittest.TestCase):
    def test_fix_import_path_invalid_case(self):
        cases = [
            {"root": None, "filepath": None, "import_path": None},
            {"root": "", "filepath": "", "import_path": ""},
        ]
        for case in cases:
            root, filepath, import_path = (
                case["root"],
                case["filepath"],
                case["import_path"],
            )
            with self.assertRaises(AssertionError):
                vfs.FixImportPath(root, filepath, import_path)

    def test_fix_import_path_normal_case(self):
        cases = [
            {
                "root": ".",
                "filepath": "path/to/app/file.k",
                "import_path": ".sub",
                "expected": "path.to.app.sub",
            },
            {
                "root": ".",
                "filepath": "path/to/app/file.k",
                "import_path": "..sub",
                "expected": "path.to.sub",
            },
            {
                "root": ".",
                "filepath": "path/to/app/file.k",
                "import_path": "...sub",
                "expected": "path.sub",
            },
            {
                "root": ".",
                "filepath": "path/to/app/file.k",
                "import_path": "....sub",
                "expected": "sub",
            },
            {
                "root": ".",
                "filepath": "path/to/app/file.k",
                "import_path": ".....sub",
                "expected": "",
            },
            {
                "root": ".",
                "filepath": "path/to/app/file.k",
                "import_path": "path.to.sub",
                "expected": "path.to.sub",
            },
            {
                "root": "path/to/app/",
                "filepath": "path/to/app/file.k",
                "import_path": ".sub",
                "expected": "sub",
            },
            {
                "root": "path/to/app/",
                "filepath": "path/to/app/file.k",
                "import_path": "..sub",
                "expected": "",
            },
            {
                "root": "path/to/",
                "filepath": "path/to/app/file.k",
                "import_path": ".sub",
                "expected": "app.sub",
            },
            {
                "root": "path/to/",
                "filepath": "path/to/app/file.k",
                "import_path": "..sub",
                "expected": "sub",
            },
            {
                "root": "path/",
                "filepath": "path/to/app/file.k",
                "import_path": "..sub",
                "expected": "to.sub",
            },
            {
                "root": "path/",
                "filepath": "path/to/app/file.k",
                "import_path": ".sub",
                "expected": "to.app.sub",
            },
        ]
        for case in cases:
            root, filepath, import_path, expected = (
                case["root"],
                case["filepath"],
                case["import_path"],
                case["expected"],
            )
            self.assertEqual(
                vfs.FixImportPath(root, filepath, import_path),
                expected,
                f"Case test failed: root: {root}, filepath: {filepath}, import_path: {import_path}",
            )


class TestVfsSaveAndLoadASTPkgCache(unittest.TestCase):
    def test_ast_save_and_load(self):
        codes = ["a = 1", "b = 1"]
        pkgs = ["pkg", "pkg.pkg"]
        for code, pkg in zip(codes, pkgs):
            module = parser.ParseFile("test.k", code)
            module_load = vfs.LoadPkgCache(os.path.dirname(__file__), pkg)
            self.assertEqual(module_load, None)
            vfs.SavePkgCache(os.path.dirname(__file__), pkg, module)
            module_load = vfs.LoadPkgCache(os.path.dirname(__file__), pkg)
            self.assertIsInstance(module_load, ast.Module)

    def test_cache_loaded_expired(self):
        code = "a = 1"
        pkg = "pkg"
        module = parser.ParseFile("test.k", code)
        vfs.SavePkgCache(os.path.dirname(__file__), pkg, module)
        module_load = vfs.LoadPkgCache(os.path.dirname(__file__), pkg)
        self.assertIsInstance(module_load, ast.Module)
        pathlib.Path(__file__).parent.joinpath("pkg/pkg.k").write_text(code)
        module_load = vfs.LoadPkgCache(os.path.dirname(__file__), pkg)
        self.assertIsNone(module_load)    
        pathlib.Path(__file__).parent.joinpath("pkg/pkg.k").write_text("")


class TestVfsSaveAndLoadASTMainPkgCache(unittest.TestCase):
    def test_ast_main_save_and_load(self):
        codes = ["a = 1", "b = 1"]
        filenames = ["main_test1.k", "main_test2.k"]
        for code, filename in zip(codes, filenames):
            module = parser.ParseFile(filename, code)
            module_load = vfs.LoadMainPkgCache(os.path.dirname(__file__), filename)
            self.assertEqual(module_load, None)
            vfs.SaveMainPkgCache(os.path.dirname(__file__), filename, module)
            module_load = vfs.LoadMainPkgCache(os.path.dirname(__file__), filename)
            self.assertIsInstance(module_load, ast.Module)

    def test_main_cache_loaded_expired(self):
        code = "a = 1"
        pkg = "pkg"
        module = parser.ParseFile("test.k", code)
        k_filepath = pathlib.Path(__file__).parent.joinpath("pkg/pkg.k")
        vfs.SaveMainPkgCache(os.path.dirname(__file__), str(k_filepath), module)
        module_load = vfs.LoadMainPkgCache(os.path.dirname(__file__), str(k_filepath))
        self.assertIsInstance(module_load, ast.Module)
        k_filepath.write_text(code)
        module_load = vfs.LoadMainPkgCache(os.path.dirname(__file__), str(k_filepath))
        self.assertIsNone(module_load)    
        k_filepath.write_text("")

    def test_kcl_mod_with_main_pkg_save(self):
        work_dir = pathlib.Path(__file__).parent
        main_file = str(work_dir / "main.k")
        # Save main package cache
        parser.LoadProgram(main_file, work_dir=str(work_dir))
        # Load main package cache
        parser.LoadProgram(main_file, work_dir=str(work_dir))


class TestCacheInfo(unittest.TestCase):
    def test_cache_info(self):
        root = str(pathlib.Path(__file__).parent)
        filepath = str(pathlib.Path(__file__).parent / "pkg")
        vfs.write_info_cache({}, root, filepath)
        # Read info cache
        vfs.read_info_cache(root)

    def test_cache_info_invalid_root(self):
        root = str(pathlib.Path(__file__).parent / "err_pkg")
        self.assertEqual(vfs.read_info_cache(root), {})

    def test_cache_info_file(self):
        cases = [
            {"filepath": str(pathlib.Path(__file__).parent / "pkg"), "expected": "d41d8cd98f00b204e9800998ecf8427e"},
            {"filepath": str(pathlib.Path(__file__).parent / "pkg/pkg.k"), "expected": "d41d8cd98f00b204e9800998ecf8427e"},
            {"filepath": str(pathlib.Path(__file__).parent / "main.k"), "expected": "fd352b68bf83391284e044021cab0339"},
        ]
        for case in cases:
            filepath, expected = case["filepath"], case["expected"]
            self.assertEqual(vfs.get_cache_info(filepath), expected)


class TestVfsSaveAndLoadBytecodeCache(unittest.TestCase):
    def setUp(self):
        self.test_data_path_name = "test_data_bytecode_cache"
        self.root = str(pathlib.Path(__file__).parent.joinpath(self.test_data_path_name))
        self.main_file = str(pathlib.Path(__file__).parent.joinpath(f"{self.test_data_path_name}/main.k"))
        self.cache_path = str(pathlib.Path(__file__).parent.joinpath(f"{self.test_data_path_name}/.kclvm"))
        return super().setUp()

    def test_save_bytecode_cache(self):
        ast_program = parser.LoadProgram(self.main_file)
        kcl_program = compiler.CompileProgram(ast_program)
        self.assertEqual(kcl_program.root, self.root)
        vfs.SaveBytecodeCache(ast_program.root, ast_program, kcl_program)
        program_loaded = vfs.LoadBytecodeCache(self.root, ast_program)
        self.assertIsNotNone(program_loaded)
        if os.path.exists(self.cache_path):
            shutil.rmtree(self.cache_path)

    def test_load_bytecode_cache(self):
        pass

    def test_save_bytecode_cache_invalid_parameters(self):
        cases = [
            {"root": "", "ast_program": None, "program": None},
            {"root": self.root, "ast_program": None, "program": None},
            {"root": self.root, "ast_program": ast.Program(), "program": None},
            {"root": self.root, "ast_program": ast.Program(), "program": obj.KCLProgram()},
        ]
        for case in cases:
            root, ast_program, program = case["root"], case["ast_program"], case["program"]
            vfs.SaveBytecodeCache(root, ast_program, program)

    def test_load_bytecode_cache_invalid_parameters(self):
        cases = [
            {"root": "", "ast_program": None},
            {"root": self.root, "ast_program": None},
            {"root": self.root, "ast_program": ast.Program()},
        ]
        for case in cases:
            root, ast_program = case["root"], case["ast_program"]
            vfs.LoadBytecodeCache(root, ast_program)


if __name__ == "__main__":
    unittest.main(verbosity=2)
