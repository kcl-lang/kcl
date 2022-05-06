"""basic checker for Python code"""
import re
from typing import Pattern

import kclvm.tools.lint.message.message as message
import kclvm.kcl.ast as ast
import kclvm.tools.lint.lint.utils as utils
from kclvm.tools.lint.checkers.base_checker import BaseChecker


MSGS = {
    "C0103": (
        '%s name "%s" doesn\'t conform to %s.',
        "Invalid-name.",
        "{0} name {1} doesn't conform to {2}.",
    ),
    "C0104": (
        'Disallowed name "%s".',
        "Disallowed-name.",
        'Disallowed name "{0}."',
    ),
}


class NamingStyle:
    ANY: Pattern[str] = re.compile(".*")
    MOD_NAME_RGX: Pattern[str] = ANY
    PKG_NAME_RGX: Pattern[str] = ANY
    SCHEMA_NAME_RGX: Pattern[str] = ANY
    MIXIN_NAME_RGX: Pattern[str] = ANY
    PROTOCOL_NAME_RGX: Pattern[str] = ANY
    SCHEMA_ATTRIBUTE_RGX: Pattern[str] = ANY
    VARIABLE_RGX: Pattern[str] = ANY
    ARGUMENT_RGX: Pattern[str] = ANY
    DEFAULT_NAME_RGX: Pattern[str] = ANY

    @classmethod
    def get_regex(cls, name_type):
        return {
            "module": cls.MOD_NAME_RGX,
            "package": cls.PKG_NAME_RGX,
            "schema": cls.SCHEMA_NAME_RGX,
            "mixin": cls.MIXIN_NAME_RGX,
            "protocol": cls.PROTOCOL_NAME_RGX,
            "argument": cls.ARGUMENT_RGX,
            "variable": cls.VARIABLE_RGX,
            "schema_attribute": cls.SCHEMA_ATTRIBUTE_RGX,
        }[name_type]


class SnakeCaseStyle(NamingStyle):
    ANY: Pattern[str] = re.compile(".*")
    MOD_NAME_RGX: Pattern[str] = re.compile(r"^\$?[a-z][a-z_]*$")
    PKG_NAME_RGX: Pattern[str] = re.compile(r"^\$?[a-z][a-z_]*$")
    SCHEMA_NAME_RGX: Pattern[str] = ANY
    MIXIN_NAME_RGX: Pattern[str] = ANY
    PROTOCOL_NAME_RGX: Pattern[str] = ANY
    VARIABLE_RGX: Pattern[str] = ANY
    SCHEMA_ATTRIBUTE_RGX: Pattern[str] = ANY
    ARGUMENT_RGX: Pattern[str] = ANY
    DEFAULT_NAME_RGX: Pattern[str] = ANY


class CamelCaseStyle(NamingStyle):
    """Regex rules for camelCase naming style."""

    ANY: Pattern[str] = re.compile(".*")
    MOD_NAME_RGX: Pattern[str] = ANY
    PKG_NAME_RGX: Pattern[str] = ANY
    SCHEMA_NAME_RGX: Pattern[str] = ANY
    MIXIN_NAME_RGX: Pattern[str] = ANY
    PROTOCOL_NAME_RGX: Pattern[str] = ANY
    VARIABLE_RGX: Pattern[str] = ANY
    SCHEMA_ATTRIBUTE_RGX: Pattern[str] = re.compile(r"^\$?[a-z][a-zA-Z]*$")
    ARGUMENT_RGX: Pattern[str] = re.compile(r"^\$?[a-z][a-zA-Z]*$")
    DEFAULT_NAME_RGX: Pattern[str] = re.compile(r"^\$?[a-z][a-zA-Z]*$")


class PascalCaseStyle(NamingStyle):
    """Regex rules for PascalCase naming style."""

    ANY: Pattern[str] = re.compile(".*")
    MOD_NAME_RGX: Pattern[str] = ANY
    PKG_NAME_RGX: Pattern[str] = ANY
    SCHEMA_NAME_RGX: Pattern[str] = re.compile(r"^\$?[A-Z][a-zA-Z\d]*$")
    MIXIN_NAME_RGX: Pattern[str] = re.compile(r"^\$?[A-Z][a-zA-Z\d]*Mixin$")
    PROTOCOL_NAME_RGX: Pattern[str] = re.compile(r"^\$?[A-Z][a-zA-Z\d]*Protocol$")
    VARIABLE_RGX: Pattern[str] = ANY
    SCHEMA_ATTRIBUTE_RGX: Pattern[str] = ANY
    ARGUMENT_RGX: Pattern[str] = ANY
    DEFAULT_NAME_RGX: Pattern[str] = ANY


class UpperCaseStyle(NamingStyle):
    """Regex rules for UPPER_CASE naming style."""

    ANY: Pattern[str] = re.compile(".*")
    MOD_NAME_RGX: Pattern[str] = ANY
    PKG_NAME_RGX: Pattern[str] = ANY
    SCHEMA_NAME_RGX: Pattern[str] = ANY
    MIXIN_NAME_RGX: Pattern[str] = ANY
    PROTOCOL_NAME_RGX: Pattern[str] = ANY
    VARIABLE_RGX: Pattern[str] = ANY
    SCHEMA_ATTRIBUTE_RGX: Pattern[str] = ANY
    ARGUMENT_RGX: Pattern[str] = ANY
    DEFAULT_NAME_RGX: Pattern[str] = re.compile(r"^\$?[^\W\da-z_][^\Wa-z]*$")


class AnyStyle(NamingStyle):
    pass


NAMING_STYLES = {
    "snake_case": SnakeCaseStyle,
    "camelCase": CamelCaseStyle,
    "PascalCase": PascalCaseStyle,
    "UPPER_CASE": UpperCaseStyle,
    "ANY": AnyStyle,
}


KNOWN_NAME_TYPES = {
    "module",
    "package",
    "schema",
    "mixin",
    "protocol",
    "argument",
    "variable",
    "schema_attribute",
}


class BasicChecker(BaseChecker):
    def __init__(self, linter) -> None:
        super().__init__(linter)
        self.name = "BaseCheck"
        self.code = None
        self.MSGS = MSGS
        self.module = None
        self.naming_rules = None
        self.bad_names = None
        self.prog = None

    def reset(self) -> None:
        self.msgs.clear()
        self.code = None
        self.module = None
        self.naming_rules = None
        self.bad_names = None

    def get_module(self, prog: ast.Program, code: str) -> None:
        assert isinstance(prog, ast.Program)
        assert code is not None
        self.prog = prog
        self.module = prog.pkgs["__main__"][0]
        self.code = code
        self.naming_rules = self._create_naming_rules()
        self.bad_names = self.options.bad_names

    def check(self, prog: ast.Program, code: str) -> None:
        assert isinstance(prog, ast.Program)
        assert code is not None
        self.reset()
        self.get_module(prog, code)
        self.walk(self.module)

    def _create_naming_rules(self):
        regexps = {}
        for name_type in KNOWN_NAME_TYPES:
            # naming rgx in config, .e.g  module-rgx : [^\W\dA-Z][^\WA-Z]+$
            custom_regex_setting_name = f"{name_type}_rgx"
            if getattr(self.options, custom_regex_setting_name):
                regexps[name_type] = re.compile(
                    getattr(self.options, custom_regex_setting_name)
                )
            else:
                # naming rgx in config, .e.g  module-rgx : [^\W\dA-Z][^\WA-Z]+$
                naming_style_name = f"{name_type}_naming_style"
                regexps[name_type] = NAMING_STYLES[
                    getattr(self.options, naming_style_name)
                ].get_regex(name_type)
        return regexps

    def _check_name(self, name: str, name_type: str):
        return self.naming_rules[name_type].search(name)

    def _disallowed_name(self, name: str):
        return name in self.bad_names

    def _get_name_style_or_rgx(self, name_type):
        name_rgx = f"{name_type}_rgx"
        name_style = f"{name_type}_naming_style"
        if getattr(self.options, name_rgx):
            return getattr(self.options, name_rgx)
        else:
            return getattr(self.options, name_style) + " naming style"

    def walk_RuleStmt(self, t: ast.RuleStmt) -> None:
        assert isinstance(t, ast.RuleStmt)
        if not self._check_name(t.name, "schema"):
            name_style_or_rgx = self._get_name_style_or_rgx("schema")
            self.msgs.append(
                message.Message(
                    "C0103",
                    self.module.filename,
                    MSGS["C0103"][0] % ("Schema", t.name, name_style_or_rgx),
                    utils.get_source_code(
                        self.module.filename, t.name_node.line, self.code
                    ),
                    (t.name_node.line, t.name_node.column),
                    ["Schema", t.name, name_style_or_rgx],
                )
            )
        self.generic_walk(t)

    def walk_SchemaStmt(self, t: ast.SchemaStmt) -> None:
        assert isinstance(t, ast.SchemaStmt)
        if t.is_mixin:
            if not self._check_name(t.name, "mixin"):
                name_style_or_rgx = self._get_name_style_or_rgx("mixin")
                self.msgs.append(
                    message.Message(
                        "C0103",
                        self.module.filename,
                        MSGS["C0103"][0] % ("Mixin", t.name, name_style_or_rgx),
                        utils.get_source_code(
                            self.module.filename, t.name_node.line, self.code
                        ),
                        (t.name_node.line, t.name_node.column),
                        ["Mixin", t.name, name_style_or_rgx],
                    )
                )
        elif t.is_protocol:
            if not self._check_name(t.name, "protocol"):
                name_style_or_rgx = self._get_name_style_or_rgx("schema")
                self.msgs.append(
                    message.Message(
                        "C0103",
                        self.module.filename,
                        MSGS["C0103"][0] % ("Protocol", t.name, name_style_or_rgx),
                        utils.get_source_code(
                            self.module.filename, t.name_node.line, self.code
                        ),
                        (t.name_node.line, t.name_node.column),
                        ["Protocol", t.name, name_style_or_rgx],
                    )
                )
        else:
            if not self._check_name(t.name, "schema"):
                name_style_or_rgx = self._get_name_style_or_rgx("schema")
                self.msgs.append(
                    message.Message(
                        "C0103",
                        self.module.filename,
                        MSGS["C0103"][0] % ("Schema", t.name, name_style_or_rgx),
                        utils.get_source_code(
                            self.module.filename, t.name_node.line, self.code
                        ),
                        (t.name_node.line, t.name_node.column),
                        ["Schema", t.name, name_style_or_rgx],
                    )
                )
        self.generic_walk(t)

    def walk_Arguments(self, t: ast.Arguments) -> None:
        assert isinstance(t, ast.Arguments)
        for arg in t.args:
            for name in arg.name_nodes:
                if not self._check_name(name.value, "argument"):
                    name_style_or_rgx = self._get_name_style_or_rgx("argument")
                    self.msgs.append(
                        message.Message(
                            "C0103",
                            self.module.filename,
                            MSGS["C0103"][0]
                            % ("Argument", name.value, name_style_or_rgx),
                            utils.get_source_code(
                                self.module.filename, t.line, self.code
                            ),
                            (t.line, t.column),
                            ["Argument", name.value, name_style_or_rgx],
                        )
                    )
        self.generic_walk(t)

    def walk_SchemaAttr(self, t: ast.SchemaAttr) -> None:
        assert isinstance(t, ast.SchemaAttr)
        if not self._check_name(t.name, "schema_attribute"):
            name_style_or_rgx = self._get_name_style_or_rgx("schema_attribute")
            self.msgs.append(
                message.Message(
                    "C0103",
                    self.module.filename,
                    MSGS["C0103"][0] % ("Schema attribute", t.name, name_style_or_rgx),
                    utils.get_source_code(self.module.filename, t.line, self.code),
                    (t.line, t.column),
                    ["Schema attribute", t.name, name_style_or_rgx],
                )
            )
        self.generic_walk(t)

    def walk_AssignStmt(self, t: ast.AssignStmt) -> None:
        assert isinstance(t, ast.AssignStmt)
        for target in t.targets:
            for name in target.name_nodes:
                if not self._check_name(name.value, "variable"):
                    name_style_or_rgx = self._get_name_style_or_rgx("variable")
                    self.msgs.append(
                        message.Message(
                            "C0103",
                            self.module.filename,
                            MSGS["C0103"][0]
                            % ("Variable", name.value, name_style_or_rgx),
                            utils.get_source_code(
                                self.module.filename, t.line, self.code
                            ),
                            (name.line, name.column),
                            ["Variable", name.value, name_style_or_rgx],
                        )
                    )
        self.generic_walk(t)

    def walk_Name(self, t: ast.Name) -> None:
        assert isinstance(t, ast.Name)
        if self._disallowed_name(t.value):
            self.msgs.append(
                message.Message(
                    "C0104",
                    self.module.filename,
                    MSGS["C0104"][0] % t.value,
                    utils.get_source_code(self.module.filename, t.line, self.code),
                    (t.line, t.column),
                    [t.value],
                )
            )
