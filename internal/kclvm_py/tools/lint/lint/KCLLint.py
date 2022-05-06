"""
KCLLinter class controls all inspection processes of lint: loading config, checking and generating reports.

The workflow of KCLLinter is as follows:
1. Load config.
2. Find all KCL files under the 'path' from CLI arguments, and parse them to ast.Program.
3. Register checker and reporter according to config
4. Distribute ast to each checker for checking, and generate Message，which represents the result of check.
5. Linter collects Messages from all checkers.
6. Distribute Message to each reporter as output
┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   KCLLinter                                                                 │
│                                                                                                             │
│      ┌───────────┐                  ┌─────────────────────────────────────────────────────────────────┐     │
│      │  KCL file │                  │                             Checker                             │     │
│      └───────────┘                  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │     │
│            ↓                        │  │  importChecker  │  │  schemaChecker  │  │       ...       │  │     │
│      ┌───────────┐                  │  │  ┌───────────┐  │  │  ┌───────────┐  │  │  ┌───────────┐  │  │     │
│      │  ast.Prog │       →          │  │  │  Message  │  │  │  │  Message  │  │  │  │  Message  │  │  │     │
│      └───────────┘                  │  │  └───────────┘  │  │  └───────────┘  │  │  └───────────┘  │  │     │
│                                     │  │  ┌───────────┐  │  │  ┌───────────┐  │  │  ┌───────────┐  │  │     │
│                                     │  │  │  Message  │  │  │  │  Message  │  │  │  │  Message  │  │  │     │
│                                     │  │  └───────────┘  │  │  └───────────┘  │  │  └───────────┘  │  │     │
│      ┌──────────────────────┐       │  │  ┌───────────┐  │  │  ┌───────────┐  │  │  ┌───────────┐  │  │     │
│      │      Config          │       │  │  │    ...    │  │  │  │    ...    │  │  │  │    ...    │  │  │     │
│      │                      │       │  │  └───────────┘  │  │  └───────────┘  │  │  └───────────┘  │  │     │
│      │   1 config           │       │  └─────────────────┘  └─────────────────┘  └─────────────────┘  │     │
│      │   2 .kcllint         │       └─────────────────────────────────────────────────────────────────┘     │
│      │   3 default_config   │                                                                               │
│      │                      │                                        ↓                                      │
│      │                      │       msgs_map -> MessageID: count                                            │
│      └──────────────────────┘       msgs ->    ┌────────────────────────────────────────────────────┐       │
│                                                │  ┌───────────┐  ┌───────────┐  ┌───────────┐       │       │
│                                                │  │  Message  │  │  Message  │  │  Message  │       │       │
│                                                │  └───────────┘  └───────────┘  └───────────┘       │       │
│                                                └────────────────────────────────────────────────────┘       │
│                                                                                                             │
│                                                                      ↓                                      │
│                                     ┌─────────────────────────────────────────────────────────────────┐     │
│                                     │                              Reporter                           │     │
│                                     │  ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐     │     │
│                                     │  │  stdout   │  │   sarif   │  │   file    │  │   ...     │     │     │
│                                     │  └───────────┘  └───────────┘  └───────────┘  └───────────┘     │     │
│                                     └─────────────────────────────────────────────────────────────────┘     │
│                                                                                                             │
│                                                                                                             │
│                                                                                                             │
└─────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
"""
import os
import glob
import ruamel.yaml as yaml
from typing import Dict, Any, Optional, List

import kclvm.compiler.parser.parser as parser
from kclvm.tools.lint.checkers import *
import kclvm.tools.lint.reporters.base_reporter as base_reporter
import kclvm.tools.lint.reporters.stdout_reporter as stdout_reporter
import kclvm.tools.lint.reporters.file_reporter as file_reporter
import kclvm.tools.lint.reporters.sarif_reporter as sarif_reporter
import kclvm.kcl.error.kcl_error as error
import kclvm.tools.lint.message.message as message
import kclvm.tools.lint.lint.exceptions as exceptions
import kclvm.kcl.info as kcl_info
import kclvm.kcl.ast as ast

LINT_CONFIG_SUFFIX = ".kcllint"
DEFAULT_CONFIG = {
    "check_list": ["import", "misc", "basic"],
    "ignore": [],
    "max_line_length": 200,
    "output": ["stdout"],
    "output_path": None,
    "module_naming_style": "ANY",
    "package_naming_style": "ANY",
    "schema_naming_style": "PascalCase",
    "mixin_naming_style": "PascalCase",
    "protocol_naming_style": "PascalCase",
    "argument_naming_style": "camelCase",
    "variable_naming_style": "ANY",
    "schema_attribute_naming_style": "ANY",
    "module_rgx": None,
    "package_rgx": None,
    "schema_rgx": None,
    "mixin_rgx": None,
    "protocol_rgx": None,
    "argument_rgx": None,
    "variable_rgx": None,
    "schema_attribute_rgx": None,
    "bad_names": ["foo", "bar", "baz", "toto", "tata", "tutu", "I", "l", "O"],
}

MSGS = {"E0999": ("Parse failed:%s.", "Parse failed.", "Parse failed:'{0}'.")}
PARSE_FAILED_MSG_ID = "E0999"


class CheckerFactory:
    @staticmethod
    def get_checker(checker: str, linter=None) -> Optional[base_checker.BaseChecker]:
        if checker == "import":
            return imports.ImportsChecker(linter)
        elif checker == "misc":
            return misc.MiscChecker(linter)
        elif checker == "basic":
            return basic.BasicChecker(linter)
        else:
            raise exceptions.InvalidCheckerError(checker)


class ReporterFactory:
    @staticmethod
    def get_reporter(
        reporter: str, linter=None
    ) -> Optional[base_reporter.BaseReporter]:
        if reporter == "stdout":
            return stdout_reporter.STDOUTReporter(linter)
        elif reporter == "file":
            return file_reporter.FileReporter(linter)
        elif reporter == "sarif":
            return sarif_reporter.SARIFReporter(linter)
        else:
            raise exceptions.InvalidReporterError(reporter)


class LinterConfig:
    def __init__(self):
        self.check_list = ["import", "misc", "basic"]
        self.ignore = []
        self.max_line_length = 200
        self.output = ["stdout"]
        self.output_path = None
        self.module_naming_style = "ANY"
        self.package_naming_style = "ANY"
        self.schema_naming_style = "PascalCase"
        self.mixin_naming_style = "PascalCase"
        self.protocol_naming_style = "PascalCase"
        self.argument_naming_style = "camelCase"
        self.variable_naming_style = "ANY"
        self.schema_attribute_naming_style = "ANY"
        self.module_rgx = None
        self.package_rgx = None
        self.schema_rgx = None
        self.mixin_rgx = None
        self.protocol_rgx = None
        self.argument_rgx = None
        self.variable_rgx = None
        self.schema_attribute_rgx = None
        self.bad_names = ["foo", "bar", "baz", "toto", "tata", "tutu", "I", "l", "O"]

    def update(self, config: {}):
        for k, v in config.items():
            if hasattr(self, k):
                self.__setattr__(k, v)


class KCLLinter:
    def __init__(
        self,
        *path: str,
        config: Dict[str, Any] = None,
        k_code_list: List[str] = None,
    ) -> None:
        self.path = path or []
        self.k_code_list = k_code_list or []
        self.file_list = []
        self.programs_list = []
        self.checkers = []
        self.reporters = []
        self.config = LinterConfig()
        self.msgs = []
        self.MSGS = MSGS
        self.msgs_map = {}

        path_list = [x for x in path]
        for i, s in enumerate(path_list):
            s = os.path.abspath(s)
            if os.path.isfile(s):
                self.file_list.append(s)
            elif os.path.isdir(s):
                self.file_list += glob.glob(
                    os.path.join(s, "**", kcl_info.KCL_FILE_PATTERN),
                    recursive=True,
                )
            else:
                raise FileNotFoundError(s)

        self._load_config(config)

    def reset(self):
        self.reporters.clear()
        self.checkers.clear()
        self.MSGS = MSGS
        self.msgs.clear()
        self.msgs_map = {}

    def run(self) -> None:
        self.reset()
        self._register_checkers(self.config.check_list)
        self._register_reporters(self.config.output)
        self._get_programs(self.file_list, self.k_code_list)
        self._check(self.programs_list, self.checkers, self.k_code_list)
        self._display()

    def check(self) -> None:
        self.reset()
        self._register_checkers(self.config.check_list)
        self._register_reporters(self.config.output)
        self._get_programs(self.file_list, self.k_code_list)
        self._check(self.programs_list, self.checkers, self.k_code_list)

    def _load_config(self, config) -> None:
        for s in self.path:
            if os.path.isfile(s):
                kcllint_path = os.path.join(
                    os.path.abspath(os.path.dirname(s)), LINT_CONFIG_SUFFIX
                )
            elif os.path.isdir(s):
                kcllint_path = os.path.join(os.path.abspath(s), LINT_CONFIG_SUFFIX)
            if os.path.isfile(kcllint_path):
                with open(kcllint_path, "r", encoding="utf-8") as f:
                    kcllint_config = f.read()
                    self.config.update(yaml.safe_load(kcllint_config))
                break
        if config:
            self.config.update(config)

    def _register_checker(self, checker: base_checker.BaseChecker) -> None:
        self.checkers.append(checker)
        if hasattr(checker, "MSGS"):
            self.MSGS.update(checker.MSGS)

    def _register_checkers(self, checkers: List[str]) -> None:
        factory = CheckerFactory()
        for s in checkers:
            self._register_checker(factory.get_checker(s, self))

    def _register_reporters(self, reporters: List[str]) -> None:
        if not reporters or len(reporters) == 0:
            raise exceptions.EmptyReporterError
        if "file" in reporters or "sarif" in reporters:
            assert self.config.output_path, "Without ouput file path"
        factory = ReporterFactory()
        self.reporters = [factory.get_reporter(s, self) for s in reporters]

    def _get_programs(
        self, file_list: List[str], k_code_list: List[str]
    ) -> List[ast.Program]:
        for i, file in enumerate(file_list):
            _code = k_code_list[i] if (i < len(k_code_list)) else None
            _k_code_list = [_code] if _code else None
            try:
                prog = parser.LoadProgram(file, k_code_list=_k_code_list)
                self.programs_list.append(prog)
            except error.KCLException as err:
                if not _code:
                    with open(err.filename) as f:
                        _code = f.read()
                source_line = _code.split("\n")[err.lineno - 1]
                msg = message.Message(
                    PARSE_FAILED_MSG_ID,
                    err.filename,
                    MSGS[PARSE_FAILED_MSG_ID][0] % err.name,
                    source_line,
                    (err.lineno, err.colno),
                    [err.name],
                )
                if (msg not in self.msgs) and (
                    PARSE_FAILED_MSG_ID not in self.config.ignore
                ):
                    self.msgs.append(msg)
                    self.msgs_map[PARSE_FAILED_MSG_ID] = (
                        self.msgs_map.setdefault(PARSE_FAILED_MSG_ID, 0) + 1
                    )

    def _check(
        self,
        progs: List[ast.Program],
        checkers: List[base_checker.BaseChecker],
        k_code_list: List[str],
    ):
        for i, prog in enumerate(progs):
            if i < len(k_code_list):
                _code = k_code_list[i]
            else:
                module = prog.pkgs["__main__"][0]
                with open(module.filename) as f:
                    _code = f.read()
            for checker in checkers:
                checker.check(prog, _code)
                # collect msgs to linter
                for msg in checker.msgs:
                    if msg.msg_id in self.config.ignore:
                        continue
                    if msg not in self.msgs:
                        self.msgs.append(msg)
                        self.msgs_map[msg.msg_id] = (
                            self.msgs_map.setdefault(msg.msg_id, 0) + 1
                        )

    def _display(self) -> None:
        for reporter in self.reporters:
            reporter.display()


def kcl_lint(*path: str, config: Dict[str, Any] = None) -> List[message.Message]:
    """
    Check kcl files or all kcl files in dirs
    :param path: str, path of a kcl file or dir
    :param config: Dict[str, Any], config of lint
    :return: List[Message] result of lint check
    """
    lint = KCLLinter(*path, config=config)
    lint.check()
    return lint.msgs


def kcl_lint_code(
    *path: str,
    k_code_list: List[str],
    config: Dict[str, Any] = None,
) -> List[message.Message]:
    """
    Check individual code of string type or some code in kcl file
    if pararm:file not None, file must be a path of kcl file and code should be part of the file,
    .e.g select some code for checking in ide
    :param k_code_list: code of string type or some code in kcl file
    :param config: Dict[str, Any], config of lint
    :param path: path of kcl file
    :return: List[Message] result of lint check
    """
    assert len(k_code_list) > 0
    lint = KCLLinter(*path, config=config, k_code_list=k_code_list)
    lint.check()
    return lint.msgs
