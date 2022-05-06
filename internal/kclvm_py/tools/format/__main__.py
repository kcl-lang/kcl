import sys
import argparse
import traceback

from kclvm.tools.format.format import kcl_fmt


def kcl_format_main():
    """KCL format tool CLI entry point"""
    parser = argparse.ArgumentParser(prog="kcl-fmt")
    parser.add_argument(
        dest="file",
        type=str,
        help="Input file or path name for formatting",
    )
    parser.add_argument(
        "-R",
        "--recursive",
        dest="recursive",
        help="Iterate through subdirectories recursively",
        action="store_true",
        required=False,
    )
    parser.add_argument(
        "-w",
        "--std-output",
        dest="std_output",
        help="Whether to output format to stdout",
        action="store_true",
        required=False,
    )
    parser.add_argument(
        "-d",
        "--debug",
        dest="debug_mode",
        help="Run in debug mode (for developers only)",
        action="store_true",
        required=False,
    )
    args = parser.parse_args()
    if len(sys.argv) == 1:
        parser.print_help(sys.stdout)
        sys.exit(0)

    try:
        kcl_fmt(args.file, is_stdout=args.std_output, recursively=args.recursive)
    except Exception as err:
        if args.debug_mode:
            print(traceback.format_exc())
        else:
            print(f"{err}")


if __name__ == "__main__":
    kcl_format_main()
