# Copyright 2020 The KCL Authors. All rights reserved.
"""
kcl-doc parses KCL source code - including comments - and produces documentation as HTML or plain text.
"""
import argparse
import sys

import kclvm.kcl.error as kcl_error
import kclvm.tools.docs.doc as doc
import kclvm.tools.docs.formats as doc_formats


class DocGenMeta:
    DEFAULT_SOURCE_FORMAT = "YAML"
    DEFAULT_OUTPUT_DIR = "kcl_doc"
    I18N_FORMAT_DESC = "i18n file format, support YAML, JSON"
    DEFAULT_DOC_FILE_FORMAT = doc_formats.KCLDocFormat.MARKDOWN
    DEFAULT_LOCALE = "en"
    KCL_FILE_DESC = "KCL file paths. If there's more than one files to generate, separate them by space"
    DOC_FORMAT_DESC = (
        "Doc file format, support YAML, JSON and MARKDOWN. Defaults to MARKDOWN"
    )
    LOCALE_DESC = "I18n locale, e.g.: zh, zh_CN, en, en_AS. Defaults to en"
    OUTPUT_DIR_DESC = (
        f"Specify the output directory. Defaults to ./{DEFAULT_OUTPUT_DIR}"
    )
    REPO_URL_DESC = "The source code repository url. It will displayed in the generated doc to link to the source code."
    I18N_INPUT_PATH_DESC = (
        "The i18n input file path. It can be a path to an i18n file when generating doc for a single kcl file, "
        "or a path to a directory that contains i18n files when generating docs for multipule kcl files. "
        "The program will search for the i18n input file according to the locale when generating docs. "
        "If i18n file exists, use it instead of source file to generate the doc"
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(prog="kcl-doc")
    subparsers = parser.add_subparsers(
        dest="kcl_doc_subcmd_name", help="kcl doc sub commands"
    )

    parser_init = subparsers.add_parser(
        "init-i18n", help="init the i18n source doc files of the given KCL files"
    )
    parser_init.add_argument(
        dest="kcl_files", metavar="files", help=DocGenMeta.KCL_FILE_DESC, nargs="*"
    )
    parser_init.add_argument(
        "--format",
        dest="format",
        metavar=DocGenMeta.DEFAULT_SOURCE_FORMAT,
        default=DocGenMeta.DEFAULT_SOURCE_FORMAT,
        type=str,
        help=DocGenMeta.I18N_FORMAT_DESC,
    )
    parser_init.add_argument(
        "-o",
        "-od" "--output-dir",
        dest="output",
        default=DocGenMeta.DEFAULT_OUTPUT_DIR,
        type=str,
        help=DocGenMeta.OUTPUT_DIR_DESC,
        required=False,
    )
    parser_init.add_argument(
        "--i18n-locale",
        dest="locale",
        default=DocGenMeta.DEFAULT_LOCALE,
        help=DocGenMeta.LOCALE_DESC,
    )

    parser_info = subparsers.add_parser(
        "info-i18n", help="show an i18n source doc of a kcl file"
    )
    parser_info.add_argument(
        dest="kcl_files", metavar="files", help=DocGenMeta.KCL_FILE_DESC, nargs="*"
    )
    parser_info.add_argument(
        "--format",
        dest="format",
        metavar=DocGenMeta.DEFAULT_SOURCE_FORMAT,
        default=DocGenMeta.DEFAULT_SOURCE_FORMAT,
        type=str,
        help=DocGenMeta.I18N_FORMAT_DESC,
    )
    parser_info.add_argument(
        "--i18n-locale",
        dest="locale",
        default=DocGenMeta.DEFAULT_LOCALE,
        help=DocGenMeta.LOCALE_DESC,
    )

    # TODO: kcl-doc update-i18n cmd

    parser_generate = subparsers.add_parser(
        "generate", help="generate a displayable doc file of a kcl file"
    )
    parser_generate.add_argument(
        dest="kcl_files", metavar="files", help=DocGenMeta.KCL_FILE_DESC, nargs="*"
    )
    parser_generate.add_argument(
        "--format",
        dest="format",
        metavar=DocGenMeta.DEFAULT_SOURCE_FORMAT,
        default=DocGenMeta.DEFAULT_DOC_FILE_FORMAT,
        type=str,
        help=DocGenMeta.DOC_FORMAT_DESC,
    )
    parser_generate.add_argument(
        "-o",
        "--output-path",
        dest="output",
        default=DocGenMeta.DEFAULT_OUTPUT_DIR,
        type=str,
        help=DocGenMeta.OUTPUT_DIR_DESC,
        required=False,
    )
    parser_generate.add_argument(
        "--r",
        "-R",
        "--recursive",
        dest="recursive",
        action="store_true",
        required=False,
        help="Search directory recursively",
    )
    parser_generate.add_argument(
        "--i18n-locale",
        dest="locale",
        default=DocGenMeta.DEFAULT_LOCALE,
        help=DocGenMeta.LOCALE_DESC,
    )

    parser_generate.add_argument(
        "--repo-url",
        dest="repo_url",
        help=DocGenMeta.REPO_URL_DESC,
    )

    parser_generate.add_argument(
        "--i18n-path",
        dest="i18n_path",
        help=DocGenMeta.I18N_INPUT_PATH_DESC,
    )

    parser_generate.add_argument(
        "--with-locale-suffix",
        dest="with_locale_suffix",
        action="store_true",
        default=False,
        help="if the generated doc files have the locale suffix in their filenames",
    )

    args = parser.parse_args()

    def kcl_doc_main():
        if args.kcl_doc_subcmd_name == "init-i18n":
            doc.kcl_i18n_init(args.kcl_files, args.output, args.format, args.locale)
            print("KCL i18n meta file inited.")
            sys.exit(0)

        if args.kcl_doc_subcmd_name == "info-i18n":
            doc.kcl_i18n_info(args.kcl_files, args.format, args.locale)
            sys.exit(0)

        if args.kcl_doc_subcmd_name == "generate":
            doc.kcl_doc_generate(
                args.kcl_files,
                args.output,
                args.format,
                args.locale,
                args.recursive,
                args.repo_url,
                args.i18n_path,
                args.with_locale_suffix,
            )
            print("KCL doc generated.")
            sys.exit(0)

        print("nothing to do.\n")
        help_msg = parser.format_help()
        print(f"{help_msg}")
        sys.exit(0)

    try:
        kcl_doc_main()
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
