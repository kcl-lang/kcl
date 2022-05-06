from ast import literal_eval
from copy import deepcopy
from pathlib import Path
from typing import List
import argparse as _argparse
import ruamel.yaml as _yaml
from io import StringIO

import kclvm.kcl.ast as ast
import kclvm.kcl.error as kcl
import kclvm.config

verbose = 0
"""
Print more information and intermediate representation.
"""

debug = 0
"""
Print debug information for developers.
"""

save_temps = False
"""
Save the intermediate representation files.
"""

output = None
"""
The name of the output file.
"""

input_file = None
"""
The name of the input file.
"""

current_path = None
"""
The current path where KCL is executed.
"""

disable_none = False
"""
Disable dumping values that are None.
"""

strict_range_check = False
"""
Whether perform strict numberic range check
"""

arguments = []
"""
Top level arguments.
"""

list_option_mode = 0
"""
List option mode (>= 0).
"""

disable_schema_check = False
"""
Disabel schema check
"""

options_help_message = ""
"""
Optopns help message
"""

path_selector = []
"""
The path selector
"""

overrides = []
"""
The configuration override path and value
"""

is_target_native = False
"""Whether the target is 'native'"""


def dump():
    if verbose:
        print("=== Configurations ===")
        print("verbose      = {}".format(verbose))
        print("save_temps   = {}".format(save_temps))
        print("output       = {}".format(output))
        print("disable_none = {}".format(disable_none))
        print("input_file   = {}".format(input_file))
        print("current_path = {}".format(current_path))
        print("arguments    = {}".format(arguments))
        print("strict_range_check = {}".format(strict_range_check))

        print("list_option_mode = {}".format(list_option_mode))
        print("disable_schema_check = {}".format(disable_schema_check))


def parse_config(argsdict: dict):
    """
    Parse kclvm config with cli argument dict
    """
    if not argsdict:
        return
    kclvm.config.verbose = (
        int(argsdict.get(KCLCLIFlag.VERBOSE, 0)) or kclvm.config.verbose
    )
    kclvm.config.debug = int(argsdict.get(KCLCLIFlag.DEBUG, 0)) or kclvm.config.debug
    kclvm.config.list_option_mode = (
        int(argsdict.get(KCLCLIFlag.LIST_OPTION_MODE, 0))
        or kclvm.config.list_option_mode
    )
    kclvm.config.save_temps = (
        argsdict.get(KCLCLIFlag.SAVE_TEMPS, False) or kclvm.config.save_temps
    )
    kclvm.config.strict_range_check = (
        argsdict.get(KCLCLIFlag.STRICT_RANGE_CHECK) or kclvm.config.strict_range_check
    )
    kclvm.config.disable_none = (
        argsdict.get(KCLCLIFlag.DISABLE_NONE, False) or kclvm.config.disable_none
    )
    kclvm.config.path_selector = argsdict.get(
        KCLCLIFlag.PATH_SELECTOR, kclvm.config.path_selector
    )
    kclvm.config.overrides = argsdict.get(KCLCLIFlag.OVERRIDES, kclvm.config.overrides)
    kclvm.config.input_file = (
        argsdict.get(KCLCLIFlag.FILES)
        or argsdict.get(KCLCLIFlag.FILE)
        or kclvm.config.input_file
    )
    kclvm.config.output = (
        argsdict.get(KCLCLIFlag.OUTPUT)
        if argsdict.get(KCLCLIFlag.OUTPUT)
        else kclvm.config.output
    )
    kclvm.config.arguments += argsdict.get(KCLCLIFlag.ARGUMENT, [])


def _deal_key_value(values, keys, vals):
    is_valid_value = False
    _k, _v = None, None
    try:
        _k, _v = values
        _v = literal_eval(_v)
    except Exception:
        pass
    # _v is a string type, and avoid quotation ' and " mark escape error
    _v = _v.replace("'", "\\'").replace('"', '\\"') if isinstance(_v, str) else _v
    # the _v can only be KCL internal type
    if _v is None or isinstance(_v, (bool, int, float, str, list, dict)):
        is_valid_value = True
    if _k is None or not isinstance(_k, str) or _k.strip() == "":
        kcl.report_exception(
            err_type=kcl.ErrType.IllegalArgumentError_TYPE,
            arg_msg=f"Invalid option name: '{_k}'. should be a non-empty string",
        )
    if not is_valid_value:
        kcl.report_exception(
            err_type=kcl.ErrType.IllegalArgumentError_TYPE,
            arg_msg=f"Invalid option value: '{_object_to_yaml_str(_v)}' for option(\"{_k}\").",
        )
    keys.append(_k)
    vals.append(_v)


class KCLCLIFlag:
    FILE = "file"
    FILES = "files"
    DISABLE_NONE = "disable_none"
    DEBUG = "debug"
    STRICT_RANGE_CHECK = "strict_range_check"
    OUTPUT = "output"
    VERBOSE = "verbose"
    SAVE_TEMPS = "save_temps"
    ARGUMENT = "argument"
    PATH_SELECTOR = "path_selector"
    OVERRIDES = "overrides"
    LIST_OPTION_MODE = "list_option_mode"


class KCLTopLevelArgumentAction(_argparse.Action):
    """KCL CLI top level argument argparse action"""

    def __call__(self, parser, namespace, values, option_string=None):
        invalid_arg_msg = 'Invalid value for option "--argument(-D)"'
        split_values = values.split("=")
        _keys, _vals = [], []
        if len(split_values) == 2 and str(split_values[1]).strip() != "":
            try:
                _deal_key_value(split_values, _keys, _vals)
            except Exception as err:
                err_msg = err.arg_msg if isinstance(err, kcl.KCLException) else err
                kcl.report_exception(
                    err_type=kcl.ErrType.IllegalArgumentError_TYPE,
                    arg_msg=f"{invalid_arg_msg}: {err_msg}",
                )
        else:
            kcl.report_exception(
                err_type=kcl.ErrType.IllegalArgumentError_TYPE,
                arg_msg=f"{invalid_arg_msg}: should be in <name>=<value> pattern, got: {values}",
            )
        args = getattr(namespace, self.dest)
        args = [] if args is None else deepcopy(args)
        args += [(_k, _v) for _k, _v in zip(_keys, _vals)]
        setattr(namespace, self.dest, args)


class KCLPathSelectorAction(_argparse.Action):
    """KCL path selector action"""

    def __call__(self, parser, namespace, values, option_string=None):
        def report_exception():
            kcl.report_exception(
                err_type=kcl.ErrType.IllegalArgumentError_TYPE,
                arg_msg=f"Invalid value for option \"--path-selector(-S)\": '{values}'. path selector should be in <pkg>:<identifier> pattern",
            )

        split_values = values.split(":", 1)
        if len(split_values) > 2:
            report_exception()
        split_values = split_values if len(split_values) == 2 else ["", *split_values]
        args = getattr(namespace, self.dest)
        args = [] if args is None else deepcopy(args)
        args += [split_values]
        setattr(namespace, self.dest, args)


class KCLOverrideAction(_argparse.Action):
    """KCL path override action"""

    def __call__(self, parser, namespace, values, option_string=None):
        def report_exception():
            kcl.report_exception(
                err_type=kcl.ErrType.IllegalArgumentError_TYPE,
                arg_msg=f"Invalid value for option \"--override(-O)\": '{values}'. the override expr should be in <the variable path selector>=<value> pattern",
            )

        args = getattr(namespace, self.dest)
        if "=" in values:
            split_values = values.split("=", 1)
            paths = split_values[0].split(":", 1)
            if len(split_values) < 2 or len(paths) > 2:
                report_exception()
            paths.append(split_values[1])
            args = [] if args is None else deepcopy(args)
            paths = paths if len(paths) == 3 else ["", *paths]
            paths += [ast.OverrideAction.CREATE_OR_UPDATE]
            args += [paths]
        elif values.endswith("-"):
            paths = values[:-1].split(":", 1)
            if len(paths) > 2:
                report_exception()
            paths = paths if len(paths) == 2 else ["", *paths]
            paths += ["", ast.OverrideAction.DELETE]
            args += [paths]
        setattr(namespace, self.dest, args)


class KCLCLISettingAction:
    """KCL CLI top level setting argparse action"""

    DEFAULT_SETTING_PATH = "kcl.yaml"
    OPTION_FILE_TYPES = [".yaml", ".yml"]
    KCL_OPTION_KEY = "kcl_options"
    KCL_CLI_CONFIG_KEY = "kcl_cli_configs"

    ARGUMENTS_ITEM_KEY = "key"
    ARGUMENTS_ITEM_VALUE = "value"

    KCL_OPTION_EXAMPLE = """
kcl_options:
  - key: myArg # the option key must be a string value
    value: myArgValue"""
    INVALID_OPTION_MSG = f"invalid kcl_options value, should be list of key/value mapping. \n=== A good example will be:==={KCL_OPTION_EXAMPLE}"

    def deal_config_obj(self, data: dict, keys: list, vals: list) -> bool:
        if data is None or not isinstance(data, dict):
            kcl.report_exception(
                err_type=kcl.ErrType.IllegalArgumentError_TYPE,
                arg_msg=f"setting file content should be a mapping, got: {_object_to_yaml_str(data)}",
            )
        # Deal Arguments
        kcl_options_mapping_list = data.get(self.KCL_OPTION_KEY) or []
        if not isinstance(kcl_options_mapping_list, list):
            kcl.report_exception(
                err_type=kcl.ErrType.IllegalArgumentError_TYPE,
                arg_msg=f"{self.INVALID_OPTION_MSG}\n=== got: ===\n{_object_to_yaml_str({self.KCL_OPTION_KEY: kcl_options_mapping_list})}",
            )
        for key_value in kcl_options_mapping_list:
            if key_value is None or not isinstance(key_value, dict):
                kcl.report_exception(
                    err_type=kcl.ErrType.IllegalArgumentError_TYPE,
                    arg_msg=f"{self.INVALID_OPTION_MSG}\n=== got: ===\n{_object_to_yaml_str({self.KCL_OPTION_KEY: [key_value]})}",
                )
            _deal_key_value(
                (
                    key_value.get(self.ARGUMENTS_ITEM_KEY),
                    key_value.get(self.ARGUMENTS_ITEM_VALUE),
                ),
                keys,
                vals,
            )
        # Deal KCL CLI parameters
        data = data.get(self.KCL_CLI_CONFIG_KEY, {})
        parse_config(data)
        return True

    def deal_setting_file(self, filename, keys, vals):
        data = None
        if any(filename.endswith(t) for t in self.OPTION_FILE_TYPES):
            try:
                data = _yaml.safe_load(Path(filename).read_text(encoding="utf-8"))
            except Exception as err:
                err_msg = err.arg_msg if isinstance(err, kcl.KCLException) else err
                kcl.report_exception(
                    err_type=kcl.ErrType.IllegalArgumentError_TYPE,
                    file_msgs=[kcl.ErrFileMsg(filename=filename)],
                    arg_msg=f"Invalid yaml content of setting file:\n{err_msg}",
                )
            try:
                self.deal_config_obj(data or {}, keys, vals)
            except Exception as err:
                err_msg = err.arg_msg if isinstance(err, kcl.KCLException) else err
                kcl.report_exception(
                    err_type=kcl.ErrType.IllegalArgumentError_TYPE,
                    file_msgs=[kcl.ErrFileMsg(filename=filename)],
                    arg_msg=f"Invalid configuration in setting file:\n{err_msg}",
                )
        else:
            kcl.report_exception(
                err_type=kcl.ErrType.IllegalArgumentError_TYPE,
                file_msgs=[kcl.ErrFileMsg(filename=filename)],
                arg_msg='Invalid value for option "--setting(-Y)": the setting file should be in yaml format',
            )

    def deal(self, filenames: List[str]) -> list:
        if not filenames and not Path(self.DEFAULT_SETTING_PATH).exists():
            return []
        args = []
        for filename in filenames or [self.DEFAULT_SETTING_PATH]:
            _keys, _vals = [], []
            self.deal_setting_file(filename, _keys, _vals)
            args += [(_k, _v) for _k, _v in zip(_keys, _vals)]
        return args


def _object_to_yaml_str(obj, options=None):
    if not isinstance(obj, (list, dict)):
        return obj
    yaml_config = _yaml.YAML()
    yaml_config.indent(mapping=3, sequence=2, offset=0)
    yaml_config.allow_duplicate_keys = True

    # show null
    def my_represent_none(self, data):
        return self.represent_scalar("tag:yaml.org,2002:null", "null")

    yaml_config.representer.add_representer(type(None), my_represent_none)

    # yaml to string
    if options is None:
        options = {}
    string_stream = StringIO()
    yaml_config.dump(obj, string_stream, **options)
    output_str = string_stream.getvalue()
    string_stream.close()
    return output_str
