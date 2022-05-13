# Copyright 2021 The KCL Authors. All rights reserved.

import argparse
import sys
import pathlib

import kclvm.kcl.error as kcl_error

from .validation import validate_code


class ValidationMeta:
    DATA_FILE_DESC = "Validation data file"
    KCL_FILE_DESC = "KCL file"
    FORMAT_DESC = "Validation data file format, support YAML and JSON"


def kcl_vet_main():
    """KCL validation CLI main function."""
    parser = argparse.ArgumentParser(prog="kcl-vet")
    parser.add_argument(
        "-d",
        "--schema",
        dest="schema",
        metavar="schema",
        type=str,
        default=None,
        required=False,
    )
    parser.add_argument(
        "--format",
        dest="format",
        metavar="format",
        type=str,
        default="json",
        required=False,
        help=ValidationMeta.FORMAT_DESC,
    )
    parser.add_argument(
        "-n",
        "--attribute-name",
        dest="attribute_name",
        metavar="attribute_name",
        type=str,
        default="value",
        required=False,
    )
    parser.add_argument(
        dest="data_file",
        metavar="data_file",
        type=str,
        help=ValidationMeta.DATA_FILE_DESC,
    )
    parser.add_argument(
        dest="kcl_file",
        metavar="kcl_file",
        type=str,
        help=ValidationMeta.KCL_FILE_DESC,
    )
    args = parser.parse_args()
    if len(sys.argv) == 1:
        parser.print_help(sys.stdout)
        sys.exit(0)

    data_str = pathlib.Path(args.data_file).read_text()
    code_str = pathlib.Path(args.kcl_file).read_text()

    # Validate code
    if validate_code(
        data_str, code_str, args.schema, args.attribute_name, args.format, args.kcl_file
    ):
        print("Validate succuss!")


if __name__ == "__main__":
    try:
        kcl_vet_main()
    except kcl_error.KCLException as err:
        kcl_error.print_kcl_error_message(err, file=sys.stderr)
        sys.exit(1)
    except OSError as err:
        kcl_error.print_common_error_message(err, file=sys.stderr)
        sys.exit(1)
    except AssertionError as err:
        kcl_error.print_internal_error_message(err, file=sys.stderr)
        raise
    except Exception:
        kcl_error.print_internal_error_message(file=sys.stderr)
        raise
