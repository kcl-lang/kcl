"""Expose KCLVM command via ``python -m kclvm``, or ``kcl`` for short."""
import sys
import os
import argparse
import cProfile
import typing

import kclvm.config
import kclvm.kcl.ast as kcl_ast
import kclvm.program.exec as kclvm_exec
import kclvm.compiler.parser
import kclvm.kcl.error as kcl_error
import kclvm.internal.log as klog
import kclvm.tools.format
import kclvm.tools.docs
import kclvm.api.version
import kclvm.vm.planner as planner

from kclvm.api.object.internal import kcl_option_init_all
from kclvm.tools.list_attribute.utils import ListAttributePrinter


def Main():
    try:
        parser = argparse.ArgumentParser(
            prog="kcl", description="K Configuration Language Virtual Machine"
        )
        parser.add_argument(
            "-D",
            "--argument",
            default=[],
            action=kclvm.config.KCLTopLevelArgumentAction,
            help="Specify the top-level argument",
            required=False,
        )
        parser.add_argument(
            "-S",
            "--path-selector",
            default=[],
            action=kclvm.config.KCLPathSelectorAction,
            help="Specify the path selector",
            required=False,
        )
        parser.add_argument(
            "-O",
            "--overrides",
            default=[],
            action=kclvm.config.KCLOverrideAction,
            help="Specify the configuration override path and value",
            required=False,
        )
        parser.add_argument(
            "-Y",
            "--setting",
            help="Specify the command line setting file",
            nargs="*",
            required=False,
        )
        parser.add_argument(
            "-o", "--output", help="Specify the output file", required=False
        )
        parser.add_argument(
            "-n",
            "--disable-none",
            help="Disable dumping None values",
            action="store_true",
            required=False,
        )
        parser.add_argument(
            "--sort",
            help="Sort result keys",
            dest="sort_keys",
            action="store_true",
            required=False,
        )
        parser.add_argument(
            "-r",
            "--strict-range-check",
            help="Do perform strict numeric range check",
            action="store_true",
            required=False,
        )
        parser.add_argument(
            "-c",
            "--compile-only",
            help="Compile only",
            action="store_true",
            required=False,
        )
        parser.add_argument(
            "-s",
            "--save-temps",
            help="Save intermediate files",
            action="store_true",
            required=False,
        )
        parser.add_argument(
            "-v",
            "--verbose",
            help="Run in verbose mode",
            action="count",
            default=0,
            required=False,
        )
        parser.add_argument(
            "-d",
            "--debug",
            help="Run in debug mode (for developers only)",
            action="count",
            default=0,
            required=False,
        )
        parser.add_argument(
            "-p",
            "--profile",
            help="Perform profiling",
            action="store_true",
            required=False,
        )
        parser.add_argument(
            "-L",
            "--list-attributes",
            dest="show_attribute_list",
            help="Show schema attributes list",
            action="store_true",
        )
        parser.add_argument(
            "-l",
            "--list-options",
            dest="list_option_mode",
            default=0,
            action="count",
            help="Show kcl options list",
        )
        parser.add_argument(
            "-V",
            "--version",
            help="Show the kclvm version",
            action="version",
            version=f"kclvm version is {kclvm.api.version.VERSION}; "
            f"checksum: {kclvm.api.version.CHECKSUM}",
        )
        parser.add_argument("file", help="Input compile file", nargs="*")
        parser.add_argument(
            "--target",
            help="Specify the target type",
            type=str,
            default="",
            choices=["native", "wasm"],
            required=False,
        )

        args = parser.parse_args()
        if len(sys.argv) == 1:
            parser.print_help(sys.stdout)
            sys.exit(0)

        argsdict = vars(args)
        kclvm.config.current_path = os.getcwd()
        # 1. Deal KCL CLI using settings file
        kclvm.config.arguments = kclvm.config.KCLCLISettingAction().deal(
            argsdict["setting"]
        )
        # 2. Deal KCL CLI config using CLI arguments
        kclvm.config.parse_config(argsdict)

        compile_only = argsdict["compile_only"]
        target = argsdict["target"]

        if args.list_option_mode > 0:
            kclvm.config.list_option_mode = args.list_option_mode
            kclvm.config.disable_schema_check = True

        kclvm.config.dump()

        def kcl_main():
            kcl_option_init_all()
            if kclvm.config.input_file:
                files = kclvm.config.input_file
                if args.show_attribute_list:
                    for file in files:
                        ListAttributePrinter(file).print()
                    exit(0)

                overrides: typing.List[kcl_ast.CmdOverrideSpec] = []
                for x in kclvm.config.overrides:
                    if len(x) == 4:
                        overrides.append(
                            kcl_ast.CmdOverrideSpec(
                                pkgpath=x[0],
                                field_path=x[1],
                                field_value=x[2],
                                action=x[3],
                            )
                        )

                output = kclvm_exec.Run(
                    files,
                    cmd_overrides=overrides,
                    print_override_ast=len(overrides) > 0 and kclvm.config.debug,
                    target=f"{target}",
                )

                if kclvm.config.list_option_mode > 0:
                    print(kclvm.config.options_help_message)
                    exit(0)
                if not compile_only:
                    output = planner.YAMLPlanner(sort_keys=args.sort_keys).plan(
                        output.filter_by_path_selector(
                            to_kcl=not (
                                kclvm.config.is_target_native
                                or kclvm.config.is_target_wasm
                            )
                        ),
                        to_py=not (
                            kclvm.config.is_target_native or kclvm.config.is_target_wasm
                        ),
                    )
                klog.write_out(output)
            return

        if argsdict["profile"]:
            cProfile.runctx("kcl_main()", None, locals())
        else:
            kcl_main()
    except kcl_error.KCLException as err:
        if kclvm.config.debug and kclvm.config.verbose > 2:
            raise err
        kcl_error.print_kcl_error_message(err, file=sys.stderr)
        sys.exit(1)
    except OSError as err:
        if kclvm.config.debug and kclvm.config.verbose > 2:
            raise err
        kcl_error.print_common_error_message(err, file=sys.stderr)
        sys.exit(1)
    except AssertionError as err:
        kcl_error.print_internal_error_message(err, file=sys.stderr)
        raise
    except Exception:
        kcl_error.print_internal_error_message(file=sys.stderr)
        raise


if __name__ == "__main__":
    Main()
