# Copyright 2020 The KCL Authors. All rights reserved.

import argparse
import sys
import json

import kclvm.compiler.extension.plugin.plugin as plugin


def Main():
    parser = argparse.ArgumentParser(prog="kcl-plugin")

    subparsers = parser.add_subparsers(
        dest="kcl_plugin_subcmd_name", help="kcl plugin sub commands"
    )

    parser_list = subparsers.add_parser("list", help="list all plugins")

    parser_init = subparsers.add_parser("init", help="init a new plugin")

    parser_info = subparsers.add_parser("info", help="show plugin document")
    subparsers.add_parser("gendoc", help="gen all plugins document")
    subparsers.add_parser("version", help="show plugin version")
    parser_test = subparsers.add_parser("test", help="test plugin")

    parser_list.add_argument(dest="plugin_keyword", metavar="keyword", nargs="?")

    parser_init.add_argument(dest="plugin_name", metavar="name")
    parser_info.add_argument(dest="plugin_name", metavar="name", nargs="?")

    parser_test.add_argument(dest="plugin_name", metavar="name")

    args = parser.parse_args()

    if args.kcl_plugin_subcmd_name == "list":
        plugin_root = plugin.get_plugin_root()
        if not plugin_root:
            print("# plugin_root: <not found>")
            sys.exit(0)

        print(f"# plugin_root: {plugin.get_plugin_root()}")
        names = plugin.get_plugin_names()

        count = 0
        for name in names:
            if args.plugin_keyword and args.plugin_keyword not in name:
                continue
            info = plugin.get_info(name)
            print(f"{info['name']}: {info['describe']} - {info['version']}")
            count = count + 1

        if count == 0:
            print("no plugin")
            sys.exit(0)

        sys.exit(0)

    if args.kcl_plugin_subcmd_name == "init":
        plugin_root = plugin.get_plugin_root()
        if not plugin_root:
            print("# plugin_root: <not found>")
            sys.exit(0)

        names = plugin.get_plugin_names()
        if args.plugin_name in names:
            print(f"{args.plugin_name} exists")
            sys.exit(1)

        plugin.init_plugin(args.plugin_name)
        sys.exit(0)

    if args.kcl_plugin_subcmd_name == "info":
        plugin_root = plugin.get_plugin_root()
        if not plugin_root:
            print("# plugin_root: <not found>")
            sys.exit(0)

        if not args.plugin_name:
            print(f"plugin_root: {plugin.get_plugin_root()}")
            sys.exit(0)

        names = plugin.get_plugin_names()
        if args.plugin_name not in names:
            print(f"{args.plugin_name} not found")
            sys.exit(1)

        info = plugin.get_info(args.plugin_name)

        print(json.dumps(info, indent=4))
        sys.exit(0)

    if args.kcl_plugin_subcmd_name == "gendoc":
        plugin_root = plugin.get_plugin_root()
        if not plugin_root:
            print("# plugin_root: <plugin root not found>")
            sys.exit(0)

        for name in plugin.get_plugin_names():
            plugin.gendoc(name)

        sys.exit(0)

    if args.kcl_plugin_subcmd_name == "version":
        print(plugin.get_plugin_version())
        sys.exit(0)

    # TODO: Using kcl-test
    if args.kcl_plugin_subcmd_name == "test":
        names = plugin.get_plugin_names()
        if args.plugin_name not in names:
            print(f"{args.plugin_name} not found")
            sys.exit(1)

        import pytest

        pytest.main(["-x", plugin.get_plugin_root(args.plugin_name)])
        sys.exit(0)

    parser.print_help()
    plugin_root = plugin.get_plugin_root()
    sys.exit(0)


if __name__ == "__main__":
    Main()
