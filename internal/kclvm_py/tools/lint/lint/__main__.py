import argparse
import sys
import traceback
import os
import ruamel.yaml as yaml

from kclvm.tools.lint.lint.KCLLint import KCLLinter

LINT_CONFIG_SUFFIX = ".kcllint"


class LintMeta:
    KCL_FILE_DESC = "KCL file path"
    AUTO_FIX = "Auto fix"
    KCL_CONFIG_PATH = "KCL lint config path"


def kcl_lint_main() -> None:
    parser = argparse.ArgumentParser(prog="kcl-lint")
    """
    parser.add_argument(
        "--fix",
        dest="auto_fix",
        action="store_true",
        help=LintMeta.AUTO_FIX,
        required=False,
    )
    """
    parser.add_argument(
        "--config",
        dest="kcl_lint_config",
        metavar="file",
        type=str,
        help=LintMeta.KCL_CONFIG_PATH,
    )
    parser.add_argument(
        dest="kcl_files",
        metavar="file",
        type=str,
        help=LintMeta.KCL_FILE_DESC,
        nargs="+",
    )
    # Todo：add check_list, ignore, max_line_length, output to cli args
    args = parser.parse_args()
    if len(sys.argv) == 1:
        parser.print_help(sys.stdout)
        sys.exit(0)
    if args.kcl_lint_config:
        assert os.path.isfile(args.kcl_lint_config) and args.kcl_lint_config.endswith(
            LINT_CONFIG_SUFFIX
        ), "Path error, can't find '.kcllint'"
        config_path = args.kcl_lint_config
        with open(config_path, "r", encoding="utf-8") as f:
            config = yaml.safe_load(f)
        KCLLinter(*args.kcl_files, config=config).run()
    else:
        KCLLinter(*args.kcl_files).run()
    """
    if args.auto_fix:
        print("Auto fix: todo")
    """


if __name__ == "__main__":
    try:
        kcl_lint_main()
    except Exception:
        print(traceback.format_exc())
