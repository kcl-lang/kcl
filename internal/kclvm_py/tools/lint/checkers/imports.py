import os
from typing import Union

import kclvm.kcl.info as kcl_info
import kclvm.tools.lint.message.message as message
import kclvm.compiler.vfs as vfs
import kclvm.kcl.ast as ast
import kclvm.compiler.extension.plugin.plugin_model as plugin
import kclvm.compiler.extension.builtin.builtin as builtin
import kclvm.tools.lint.lint.utils as utils
from kclvm.tools.lint.checkers.base_checker import BaseChecker
import re

MSGS = {
    "E0401": ("Unable to import %s.", "Unable to import.", "Unable to import '{0}'."),
    "W0404": (
        "%s is reimported multiple times.",
        "Module reimported.",
        "'{0}' is reimported multiple times.",
    ),
    "E0406": (
        "Module import itself.",
        "Module import itself.",
        "Module import itself.",
    ),
    "W0411": (
        "%s imported but unused.",
        "Module imported but unused.",
        "'{0}' imported but unused.",
    ),
    "E0413": (
        "Import %s should be placed at the top of the module.",
        "ImportStmt is not at the top of the file.",
        "Import '{0}' should be placed at the top of the module.",
    ),
}

IMPORT_POSITION_CHECK_LIST = [
    "AssignStmt",
    "AugAssignStmt",
    "AssertStmt",
    "IfStmt",
    "TypeAliasStmt",
    "SchemaStmt",
    "RuleStmt",
]


class ImportsChecker(BaseChecker):
    def __init__(self, linter=None) -> None:
        super().__init__(linter)
        self.name = "ImportCheck"
        self.MSGS = MSGS
        self.prog = None
        self.module = None
        self.code = None
        self.root: str = None
        # for reimport check
        self.has_imported_modules = None
        # for unused import check
        self.import_names_map = None
        # for import position check
        self.import_position_check = True

    def reset(self) -> None:
        self.msgs.clear()
        self.module = None
        self.code = None
        self.root = None
        if self.import_names_map:
            self.import_names_map.clear()
        if self.has_imported_modules:
            self.has_imported_modules.clear()
        self.import_position_check = True

    def get_module(self, prog: ast.Program, code: str) -> None:
        assert isinstance(prog, ast.Program)
        assert code is not None
        self.prog = prog
        self.module = prog.pkgs["__main__"][0]
        self.code = code
        self.root = prog.root
        self.has_imported_modules = []
        self.import_names_map = {}

    def check(self, prog: ast.Program, code: str) -> None:
        assert isinstance(prog, ast.Program)
        assert code is not None
        self.reset()
        self.get_module(prog, code)
        self.walk(self.module)
        for k, v in self.import_names_map.items():
            self.msgs.append(
                message.Message(
                    "W0411",
                    self.module.filename,
                    MSGS["W0411"][0] % k,
                    utils.get_source_code(self.module.filename, v.line, self.code),
                    (v.line, v.column),
                    [k],
                )
            )

    def check_import_file_exist(self, t: ast.ImportStmt, abs_path: str) -> bool:
        assert isinstance(t, ast.ImportStmt)
        if os.path.isdir(abs_path) or os.path.isfile(
            abs_path + kcl_info.KCL_FILE_SUFFIX
        ):
            return True
        else:
            self.msgs.append(
                message.Message(
                    "E0401",
                    self.module.filename,
                    MSGS["E0401"][0] % t.path,
                    utils.get_source_code(self.module.filename, t.line, self.code),
                    (t.line, t.column),
                    [t.path],
                )
            )
            return False

    def check_import_position(self, t: ast.AST) -> None:
        assert isinstance(t, ast.AST)
        if self.import_position_check:
            if t._ast_type in IMPORT_POSITION_CHECK_LIST:
                self.import_position_check = False
        else:
            if isinstance(t, ast.ImportStmt):
                self.msgs.append(
                    message.Message(
                        "E0413",
                        self.module.filename,
                        MSGS["E0413"][0] % t.pkg_name,
                        utils.get_source_code(self.module.filename, t.line, self.code),
                        (t.line, t.column),
                        [t.pkg_name],
                    )
                )

    def check_unused_import(self, t: Union[ast.Identifier, str]) -> None:
        if isinstance(t, ast.Identifier):
            if t.get_first_name() in self.import_names_map.keys():
                self.import_names_map.pop(t.get_first_name())
        else:
            # SchemaAttr.types, A|B, [A|B], {A|B:C}
            type_list = re.split(r"[|:\[\]\{\}]", t)
            for type in type_list:
                names = type.split(".")
                first_name = names[0]
                if first_name in self.import_names_map.keys():
                    self.import_names_map.pop(first_name)

    def check_import_itself(self, t: ast.ImportStmt, abs_path: str) -> None:
        assert isinstance(t, ast.ImportStmt)
        if os.path.isdir(abs_path):
            return
        abs_path += kcl_info.KCL_FILE_SUFFIX
        # normpath: a/./b -> a/b
        if abs_path == str(os.path.normpath(self.module.filename)):
            self.msgs.append(
                message.Message(
                    "E0406",
                    self.module.filename,
                    MSGS["E0406"][0],
                    utils.get_source_code(self.module.filename, t.line, self.code),
                    (t.line, t.column),
                    [],
                )
            )

    def check_reimport(self, t: ast.ImportStmt, abs_path: str) -> None:
        assert isinstance(t, ast.ImportStmt)
        if abs_path in self.has_imported_modules:
            self.msgs.append(
                message.Message(
                    "W0404",
                    self.module.filename,
                    MSGS["W0404"][0] % t.pkg_name,
                    utils.get_source_code(self.module.filename, t.line, self.code),
                    (t.line, t.column),
                    [t.pkg_name],
                )
            )
        else:
            self.import_names_map[t.pkg_name] = t
            self.has_imported_modules.append(abs_path)

    def generic_walk(self, t: ast.AST) -> None:
        """Called if no explicit walker function exists for a node."""
        self.check_import_position(t)
        for field, value in ast.iter_fields(t):
            if isinstance(value, list):
                for v in value:
                    # IfStmt.elif_body: List[List[Stmt]]
                    if isinstance(v, list):
                        for v1 in v:
                            self.walk(v1)
                    if isinstance(v, ast.AST):
                        self.walk(v)
            elif isinstance(value, ast.AST):
                self.walk(value)

    def walk_Identifier(self, t: ast.Identifier) -> None:
        assert isinstance(t, ast.Identifier)
        self.check_unused_import(t)

    def walk_SchemaAttr(self, t: ast.SchemaAttr) -> None:
        assert isinstance(t, ast.SchemaAttr)
        self.check_unused_import(t.type_str)
        self.generic_walk(t)

    def walk_ImportStmt(self, t: ast.ImportStmt) -> None:
        assert isinstance(t, ast.ImportStmt)
        self.check_import_position(t)
        if (
            t.path.startswith(plugin.PLUGIN_MODULE_NAME)
            or t.name in builtin.STANDARD_SYSTEM_MODULES
        ):
            self.check_reimport(t, t.path)
            return
        fix_path = vfs.FixImportPath(self.root, self.module.filename, t.path).replace(
            ".", "/"
        )
        abs_path = os.path.join(self.root, fix_path)
        if self.check_import_file_exist(t, abs_path):
            self.check_import_itself(t, abs_path)
            self.check_reimport(t, abs_path)
