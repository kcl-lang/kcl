# Copyright 2021 The KCL Authors. All rights reserved.

import typing
import copy
import os
import pathlib
from dataclasses import dataclass

import kclvm.kcl.ast as ast
import kclvm.kcl.types as types
import kclvm.kcl.error as kcl_error
import kclvm.unification as unification
import kclvm.compiler.astutil as astutil
import kclvm.compiler.parser as parser
from kclvm.tools.printer import PrintAST, Config
from kclvm.tools.format import kcl_fmt_file


KCL_FEATURE_GATE_OVERRIDE_AUTO_FIX_ENV = "KCL_FEATURE_GATE_OVERRIDE_AUTO_FIX"


@dataclass
class OverrideInfo:
    pkgpath: str = None
    filename: str = None
    module: ast.Module = None

    # ---------------
    # Static members
    # ---------------

    MODIFIED = []


class OverrideTransformer(ast.TreeTransformer):
    def __init__(
        self,
        target_id: str,
        field_path: str,
        override_key: ast.Identifier,
        override_value: ast.Literal,
        override_action: ast.OverrideAction = ast.OverrideAction.CREATE_OR_UPDATE,
    ):
        super().__init__()
        self.target_id: str = target_id
        self.field_path: str = field_path
        self.override_key: ast.Identifier = override_key
        self.override_value: ast.Literal = override_value
        self.override_target_count: int = 0
        self.has_override: bool = False
        self.override_action: ast.OverrideAction = override_action

    def walk_UnificationStmt(self, t: ast.UnificationStmt):
        """ast.AST: UnificationStmt

        Parameters
        ----------
        - target: Identifier
        - value: SchemaExpr
        """
        name = t.target.names[0]
        if name != self.target_id:
            return t
        self.override_target_count = 1
        self.has_override = True
        self.walk(t.value)
        return t

    def walk_AssignStmt(self, t: ast.AssignStmt):
        if not isinstance(t.value, ast.SchemaExpr):
            return t
        self.override_target_count = 0
        for target in t.targets:
            if not isinstance(target, ast.Identifier):
                continue
            assign_target = typing.cast(ast.Identifier, target)
            if len(assign_target.names) != 1:
                continue
            if assign_target.names[0] != self.target_id:
                continue
            self.override_target_count = self.override_target_count + 1
        if self.override_target_count == 0:
            return t
        self.has_override = True
        schema_expr_old: ast.SchemaExpr = copy.deepcopy(t.value)
        schema_expr_new: ast.SchemaExpr = t.value
        self.walk_SchemaExpr(schema_expr_new)

        if len(t.targets) == 1:
            return t

        # Fix multiple assign
        assign_stmt_list = []
        for target in t.targets:
            x: ast.AssignStmt = copy.deepcopy(t)
            x.targets = [target]
            if len(target.names) == 1 and target.names[0] == self.target_id:
                x.value = schema_expr_new
            else:
                x.value = schema_expr_old
            assign_stmt_list.append(x)
        return assign_stmt_list

    def walk_SchemaExpr(self, t: ast.SchemaExpr):
        if self.override_target_count <= 0:
            return t
        if not self._find_schema_config_and_replace(
            t, self.field_path, self.override_value
        ):
            # Not exist and append an override value when the action is CREATE_OR_UPDATE
            if self.override_action == ast.OverrideAction.CREATE_OR_UPDATE:
                t.config.items.append(
                    ast.ConfigEntry(
                        key=self.override_key,
                        value=self.override_value,
                        operation=ast.ConfigEntryOperation.OVERRIDE,
                    )
                )
        self.override_target_count = 0
        return t

    def walk_SchemaStmt(self, t: ast.SchemaStmt):
        """Do not override AssignStmt in SchemaStmt"""
        return t

    def _get_config_field_paths(
        self,
        config: typing.Union[ast.SchemaExpr, ast.ConfigExpr],
    ) -> typing.Tuple[typing.List[str], typing.List[str]]:
        def _get_key_value_paths(
            key, value
        ) -> typing.Tuple[typing.List[str], typing.List[str]]:
            _paths, _paths_with_id = [], []
            if isinstance(key, ast.Identifier):
                path = key.get_name()
            elif isinstance(key, ast.StringLit):
                path = key.value
            else:
                return _paths, _paths_with_id
            _paths.append(f"{path}")
            _paths_with_id.append(f"{path}")
            value_paths, value_paths_with_id = self._get_config_field_paths(value)
            if value_paths:
                _paths.extend([f"{path}.{value_path}" for value_path in value_paths])
                _paths_with_id.extend(
                    [f"{path}|{value_path}" for value_path in value_paths_with_id]
                )
            return _paths, _paths_with_id

        paths, paths_with_id = [], []
        if isinstance(config, ast.SchemaExpr):
            for item in config.config.items:
                _paths, _paths_with_id = _get_key_value_paths(item.key, item.value)
                paths.extend(_paths)
                paths_with_id.extend(_paths_with_id)
        elif isinstance(config, ast.ConfigExpr):
            for key, value in zip(config.keys, config.values):
                _paths, _paths_with_id = _get_key_value_paths(key, value)
                paths.extend(_paths)
                paths_with_id.extend(_paths_with_id)
        return paths, paths_with_id

    def _replace_with_id_path(
        self,
        schema_config: typing.Union[ast.SchemaExpr, ast.ConfigExpr],
        path_with_id: str,
        override_value: ast.Literal,
    ) -> typing.Optional[ast.SchemaExpr]:
        if not path_with_id or not schema_config:
            return None
        parts = path_with_id.split("|")
        config = schema_config

        def _get_path_from_key(key: ast.AST) -> str:
            path = ""
            if isinstance(key, ast.Identifier):
                path = key.get_name()
            elif isinstance(key, ast.StringLit):
                path = key.value
            return path

        for i, part in enumerate(parts):
            if isinstance(config, ast.SchemaExpr):
                delete_index_list = []
                config_ref = config
                for j, item in enumerate(config.config.items):
                    path = _get_path_from_key(item.key)
                    if path == part:
                        if self.override_action == ast.OverrideAction.CREATE_OR_UPDATE:
                            if i == len(parts) - 1:
                                override_value.set_ast_position(config)
                                item.value = override_value
                            config = item.value
                        elif self.override_action == ast.OverrideAction.DELETE:
                            delete_index_list.append(j)
                        continue
                config_ref.config.items = [
                    item
                    for j, item in enumerate(config_ref.config.items)
                    if j not in delete_index_list
                ]
            elif isinstance(config, ast.ConfigExpr):
                key_value_pairs = zip(config.keys, config.values)
                delete_index_list = []
                config_ref = config
                for j, key_value in enumerate(key_value_pairs):
                    key, value = key_value
                    path = _get_path_from_key(key)
                    if path == part:
                        if self.override_action == ast.OverrideAction.CREATE_OR_UPDATE:
                            if i == len(parts) - 1:
                                override_value.set_ast_position(config)
                                config.items[j].value = override_value
                            config = value
                        elif self.override_action == ast.OverrideAction.DELETE:
                            delete_index_list.append(j)
                        continue
                config_ref.items = [
                    item
                    for j, item in enumerate(config_ref.items)
                    if j not in delete_index_list
                ]
        if override_value:
            override_value.set_ast_position(config)
        return schema_config

    def _find_schema_config_and_replace(
        self, schema_config: ast.SchemaExpr, field_path: str, value: typing.Any
    ) -> bool:
        if not schema_config:
            raise Exception("override schema config can't be None")
        # Find field_path by nested identifier
        paths, paths_with_id = self._get_config_field_paths(schema_config)
        if field_path not in paths:
            return False
        path_with_id = paths_with_id[paths.index(field_path)]
        self._replace_with_id_path(schema_config, path_with_id, value)
        return True


def ApplyOverrides(
    prog: ast.Program,
    overrides: typing.List[ast.CmdOverrideSpec],
    import_paths: typing.List[str] = None,
):
    for override in overrides or []:
        pkgpath = override.pkgpath if override.pkgpath else prog.main
        if pkgpath in prog.pkgs:
            for mx in prog.pkgs[pkgpath]:
                if FixModuleOverride(mx, override):
                    OverrideInfo.MODIFIED.append(
                        OverrideInfo(
                            pkgpath=pkgpath,
                            filename=mx.GetFileName(root=prog.root),
                            module=mx,
                        )
                    )
                ModuleAddImportPaths(mx, import_paths)
    # Override type check and to auto fix
    if os.getenv(KCL_FEATURE_GATE_OVERRIDE_AUTO_FIX_ENV):
        types.ResolveProgram(prog, types.CheckConfig(config_attr_auto_fix=True))
    # Put AST module modified deepcopy
    OverrideInfo.MODIFIED = [copy.deepcopy(m) for m in OverrideInfo.MODIFIED]


def PrintOverridesAST(is_fix: bool = True):
    """Print override AST program"""
    if OverrideInfo.MODIFIED:
        for value in OverrideInfo.MODIFIED:
            with open(value.filename, "w") as f:
                f.flush()
                os.fsync(f.fileno())
                PrintAST(value.module, f, Config(is_fix=is_fix))
            kcl_fmt_file(pathlib.Path(value.filename))


def ModuleAddImportPaths(
    m: ast.Module, import_paths: typing.List[str], ignore_exist: bool = False
) -> ast.Module:
    """Add more import paths into the AST module."""
    if not import_paths:
        return m
    import_stmt_list = []
    exist_import_set = [
        f"{stmt.path} as {stmt.asname}" if stmt.asname else stmt.path
        for stmt in m.GetImportList()
    ]
    line = 1
    for path in import_paths or []:
        if not ignore_exist and path in exist_import_set:
            continue
        import_stmt = ast.ImportStmt(line, 1)
        import_stmt.path = path
        import_stmt.name = path.rsplit(".")[-1]
        import_stmt_list.append(import_stmt)
        line += 1
    m.body = import_stmt_list + m.body


def FixModuleOverride(m: ast.Module, override: ast.CmdOverrideSpec) -> bool:
    assert m
    assert override

    ss = override.field_path.split(".")
    if len(ss) <= 1:
        return False

    target_id: str = ss[0]
    field: str = ".".join(ss[1:])
    value: str = override.field_value

    key = ast.Identifier(names=[s for s in field.split(".")], ctx=ast.ExprContext.STORE)
    val = astutil.BuildNodeFromString(value)

    transformer = OverrideTransformer(
        target_id, field, key, val, override_action=override.action
    )
    transformer.walk(m)
    return transformer.has_override


def override_file(
    file: str, specs: typing.List[str], import_paths: typing.List[str] = None
) -> bool:
    """Override and rewrite a file with override spec

    Parameters
    ----------
    file: str
        The File that need to be overridden
    specs: List[str]
        List of specs that need to be overridden.
        Each spec string satisfies the form: <pkgpath>:<field_path>=<filed_value> or <pkgpath>:<field_path>-
        When the pkgpath is '__main__', it can be omitted.

    Return
    ------
    result: bool
        Whether override is successful
    """
    overrides = [spec_str_to_override(spec) for spec in specs or []]
    program = parser.LoadProgram(
        file,
        mode=parser.ParseMode.ParseComments,
        load_packages=bool(os.getenv(KCL_FEATURE_GATE_OVERRIDE_AUTO_FIX_ENV)),
    )
    # Config unification
    program.pkgs[ast.Program.MAIN_PKGPATH] = unification.MergeASTList(
        program.pkgs[ast.Program.MAIN_PKGPATH]
    )
    OverrideInfo.MODIFIED = []
    ApplyOverrides(program, overrides, import_paths)
    PrintOverridesAST(False)
    return True


def spec_str_to_override(spec: str) -> ast.CmdOverrideSpec:
    """Override spec string to override structure"""

    def report_exception():
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.IllegalArgumentError_TYPE,
            arg_msg=f"Invalid spec format '{spec}', expected <pkgpath>:<field_path>=<filed_value> or <pkgpath>:<field_path>-",
        )

    # Create or update the override value
    if "=" in spec:
        split_values = spec.split("=", 1)
        paths = split_values[0].split(":", 1)
        if len(split_values) < 2 or len(paths) > 2:
            report_exception()
        paths.append(split_values[1])
        paths = paths if len(paths) == 3 else ["", *paths]
        return ast.CmdOverrideSpec(
            pkgpath=paths[0],
            field_path=paths[1],
            field_value=paths[2],
            action=ast.OverrideAction.CREATE_OR_UPDATE,
        )
    # Delete the override value
    elif spec.endswith("-"):
        paths = spec[:-1].split(":", 1)
        if len(paths) > 2:
            report_exception()
        paths = paths if len(paths) == 2 else ["", *paths]
        return ast.CmdOverrideSpec(
            pkgpath=paths[0],
            field_path=paths[1],
            field_value="",
            action=ast.OverrideAction.DELETE,
        )

    report_exception()
