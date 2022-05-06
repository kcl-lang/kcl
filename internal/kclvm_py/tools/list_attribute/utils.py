import os
import glob
import pathlib
from io import StringIO
from typing import List, Dict, Optional
from dataclasses import dataclass

import kclvm.compiler.parser.parser as parser
import kclvm.compiler.vfs as vfs
import kclvm.compiler.extension.plugin.plugin_model as plugin
import kclvm.compiler.extension.builtin.builtin as builtin
import kclvm.kcl.ast.ast as ast
import kclvm.kcl.info as kcl_info
import kclvm.tools.printer.printer as printer


@dataclass
class Config:
    name_len: int = 30
    type_len: int = 30
    default_len: int = 30
    final_len: int = 10
    optional_len: int = 10


def get_import_module(
    module: ast.Module, result: Dict[str, ast.Module] = None
) -> Optional[Dict[str, ast.Module]]:
    """Get all import module in a module."""
    if not module:
        return None
    assert isinstance(module, ast.Module)
    if not result:
        result = {}
    import_file_list = []
    import_stmt_list = module.GetImportList()
    work_dir = os.path.dirname(module.filename)
    root: str = vfs.GetPkgRoot(work_dir)
    if not root:
        root = work_dir
    for stmt in import_stmt_list or []:
        if (
            stmt.path.startswith(plugin.PLUGIN_MODULE_NAME)
            or stmt.name in builtin.STANDARD_SYSTEM_MODULES
        ):
            continue
        # import_path to abs_path
        fix_path = vfs.FixImportPath(root, module.filename, stmt.path).replace(".", "/")
        abs_path = os.path.join(root, fix_path)
        # Get all .k file if path is a folder
        if os.path.isdir(abs_path):
            file_glob = os.path.join(abs_path, "**", kcl_info.KCL_FILE_PATTERN)
            import_file_list += glob.glob(file_glob, recursive=True)
        else:
            abs_path += kcl_info.KCL_FILE_SUFFIX
            import_file_list.append(abs_path)
    for file in import_file_list:
        # Skip `_*.k` and `*_test.k` kcl files
        if os.path.basename(file).startswith("_"):
            continue
        if file.endswith("_test.k"):
            continue
        if file not in result:
            import_module = parser.ParseFile(file)
            result[file] = import_module
            if import_module.GetImportList():
                get_import_module(import_module, result)
    return result


def get_import_schema(
    module: ast.Module,
) -> Optional[Dict[ast.Module, List[ast.SchemaStmt]]]:
    """Get all import schema in a module."""
    if not module:
        return None
    assert isinstance(module, ast.Module)
    import_module_list = get_import_module(module).values()
    import_schema_map = {m: m.GetSchemaList() for m in import_module_list}
    return import_schema_map


class FullSchema(ast.SchemaStmt):
    """
    Schema with base schema's attr.
    todo： mixin attr
    """

    def __init__(self, schema: ast.SchemaStmt, module: ast.Module) -> None:
        super().__init__(schema.line, schema.column)
        self.self_schema = schema
        self.parent_attr: Dict[str, List[ast.SchemaAttr]] = get_parent_attr_map(
            schema, module, {}
        )

    def __str__(self):
        s = self.self_schema.name + ", attr:["
        for name in self.self_schema.GetAttrNameList():
            s += f"{name}, "
        s = s[:-2] + "],"
        for p in self.parent_attr:
            s += f" parent:{p}, attr:["
            for attr in self.parent_attr[p]:
                s += f"{attr.name}, "
            s = s[:-2] + "],"

        return s


def get_parent_attr_map(
    ori_schema: ast.SchemaStmt,
    module: ast.Module,
    result: Dict[str, List[ast.SchemaAttr]] = None,
) -> Optional[Dict[str, List[ast.SchemaAttr]]]:
    if not ori_schema or not module:
        return None
    assert isinstance(ori_schema, ast.SchemaStmt)
    assert isinstance(module, ast.Module)
    if not result:
        result = {}
    if not ori_schema.parent_name:
        return result
    else:
        # Current module and schema.
        full_schema_map: Dict[ast.Module, List[ast.SchemaStmt]] = {
            module: module.GetSchemaList()
        }
        # Import module and schemas.
        full_schema_map.update(get_import_schema(module))
        # key : module , value: List[ast.SchemaStmt]
        for key, value in full_schema_map.items():
            for schema in value:
                if schema.name == ori_schema.parent_name.get_name():
                    result[schema.name] = schema.GetAttrList()
                    if schema.parent_name:
                        get_parent_attr_map(schema, key, result)
                    break
            else:
                continue
            break
    return result


def get_full_schema_list(module: ast.Module) -> List[FullSchema]:
    """Get all FullSchema in a module"""
    schema_list = module.GetSchemaList()
    full_schema_list = [FullSchema(schema, module) for schema in schema_list]
    return full_schema_list


class ListAttributePrinter:
    def __init__(self, file: str = None, config: Config = Config()) -> None:
        self.file = file
        self.name_len = config.name_len
        self.type_len = config.type_len
        self.default_len = config.default_len
        self.final_len = config.final_len
        self.optional_len = config.optional_len
        self.module = None
        self.schema_list = None
        self.import_schema_list = None
        self.full_schema_list = None

    def build_full_schema_list(self):
        self.module = parser.ParseFile(self.file)
        self.schema_list = self.module.GetSchemaList()
        self.import_schema_list = get_import_schema(self.module)
        self.full_schema_list = get_full_schema_list(self.module)

    def print(self):
        self.build_full_schema_list()
        if self.module:
            self.print_schema_list()
            self.print_schema_structures()

    def print_schema_list(self):
        print("------------ schema list ------------")
        file_path = self.module.filename
        file_name = pathlib.Path(file_path).name
        print("Here are schemas defined in {}:".format(file_name))
        for schema in self.schema_list:
            print("- " + schema.name)
        print("Here are schemas imported to {}:".format(file_name))
        for key, value in self.import_schema_list.items():
            import_file_path = key.filename
            import_file_name = pathlib.Path(import_file_path).name
            if len(value) > 0:
                print("imported from {}".format(import_file_name))
                for schema in value:
                    print("- " + schema.name)

    def print_schema_structures(self):
        print("------------ schema structures ------------")
        for full_schema in self.full_schema_list:
            print("schema {}:".format(full_schema.self_schema.name))
            self.print_header()
            for attr in full_schema.self_schema.GetAttrList():
                self.print_schema_attr(attr)

            for key, value in full_schema.parent_attr.items():
                print("attrs inherited from {}".format(key))
                for attr in value:
                    self.print_schema_attr(attr)
            print()

    def _print_schema_attr(self, attr: ast.SchemaAttr, default: str):
        print(
            "{:<{}}{:<{}}{:<{}}{:<{}}{:<{}}".format(
                # name
                attr.name
                if len(attr.name) <= self.name_len
                else attr.name[: self.name_len - 3] + "...",
                self.name_len,
                # type
                attr.type_str
                if len(attr.type_str) <= self.type_len
                else attr.type_str[: self.type_len - 3] + "...",
                self.type_len,
                # default
                default,
                self.default_len,
                "",
                self.final_len,
                # is_optional
                "" if attr.is_optional else "Required",
                self.optional_len,
            )
        )

    def print_schema_attr(self, attr: ast.SchemaAttr):
        if not attr:
            return
        assert isinstance(attr, ast.SchemaAttr)
        if attr.value and isinstance(attr.value, ast.SchemaExpr):
            """
            Because ast node SchemaExpr is too long to print,
            when the default value of attr.value is a SchemaExpr,just print schema name,e.g.:
            schema Name:
                firstName : str
                lastName : str

            schema Person:
                name: Name = Name {
                    firstName = "hello"
                    lastName = "world"
                }
            -------------------------------------
            schema Person:
            name                type                          default  ...
            name                Name                      ->  Name{...}
            """
            default = (
                attr.type_str
                if len(attr.type_str) <= (self.default_len - 5)
                else attr.type_str[: self.default_len - 5]
            ) + "{...}"
            self._print_schema_attr(attr, default)
            return
        with StringIO() as expr:
            printer.PrintAST(attr.value, expr)
            default_str = expr.getvalue()
            if len(default_str) > self.default_len or ("\n" in default_str):
                default_str = "..."
            self._print_schema_attr(attr, default_str)

    def print_header(self):
        print(
            "{:<{}}{:<{}}{:<{}}{:<{}}{:<{}}".format(
                # name
                "name",
                self.name_len,
                # type
                "type",
                self.type_len,
                # default
                "default",
                self.default_len,
                # is_final
                "is_final",
                self.final_len,
                # is_optional
                "is_optional",
                self.optional_len,
            )
        )
