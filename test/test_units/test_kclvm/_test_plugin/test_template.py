# Copyright 2020 The KCL Authors. All rights reserved.

import kclvm.compiler.extension.plugin.template as template


hello_plugin_name = "kcl_plugin.hello"


def test_reset_plugin():
    template.get_plugin_template_code(hello_plugin_name)
    template.get_plugin_test_template_code(hello_plugin_name)
