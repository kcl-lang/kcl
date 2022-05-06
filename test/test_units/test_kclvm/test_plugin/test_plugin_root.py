# Copyright 2020 The KCL Authors. All rights reserved.

import os
import pathlib
import sys

import kclvm.compiler.extension.plugin.plugin as plugin


def test_find_plugin_root():
    os.environ["KCL_PLUGINS_ROOT"] = ""

    root1 = plugin.find_plugin_root()
    sys.stdout.write(f"root1:{root1}\n")
    if root1 is None:
        return


    root1_hello_plugin_py = ""
    if os.path.exists(f"{root1}/hello/plugin.py"):
        root1_hello_plugin_py = f"{root1}/hello/plugin.py"
        os.rename(root1_hello_plugin_py, root1_hello_plugin_py+"_test_tmp")

    root2 = plugin.find_plugin_root()
    sys.stdout.write(f"root2:{root2}\n")
    if root2 is None:
        if root1_hello_plugin_py:
            os.rename(root1_hello_plugin_py+"_test_tmp", root1_hello_plugin_py)
        return

    root2_hello_plugin_py = ""
    if os.path.exists(f"{root2}/hello/plugin.py"):
        root2_hello_plugin_py = f"{root2}/hello/plugin.py"
        os.rename(root2_hello_plugin_py, root2_hello_plugin_py+"_test_tmp")

    plugin.find_plugin_root()  # skip $HOME/.kusion/kclvm/plugins
    root3 = plugin.find_plugin_root()
    assert root3 is None

    if root1_hello_plugin_py:
        os.rename(root1_hello_plugin_py+"_test_tmp", root1_hello_plugin_py)
    if root2_hello_plugin_py:
        os.rename(root2_hello_plugin_py+"_test_tmp", root2_hello_plugin_py)

    root4 = plugin.find_plugin_root()
    assert root4 == root1


def test_plugin_version():
    plugin.init_.plugins_root = None
    assert plugin.get_plugin_version() == plugin.UNKNOWN_VERSION
    plugin.init_.plugins_root = str(pathlib.Path(__file__).parent)
    assert plugin.get_plugin_version() == plugin.UNKNOWN_VERSION
    plugin.init_.plugins_root = str(pathlib.Path(__file__).parent.joinpath("test_data"))
    assert plugin.get_plugin_version() == "test_plugin_version"
