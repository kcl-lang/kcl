# Copyright 2021 The KCL Authors. All rights reserved.

import os
import distutils.core

PWD = os.path.abspath(os.path.dirname(__file__))
kclvm_ROOT = os.path.abspath(f"{PWD}/..")

distutils.core.setup(
    name="kclvm-plugin",
    version="1.0",
    py_modules=["kclvm_plugin", "kclvm_runtime"],
    ext_modules=[
        distutils.core.Extension(
            "_kclvm_plugin",
            [f"{PWD}/kclvm_plugin.cpp", f"{PWD}/kclvm_plugin_wrap.cxx"],
            include_dirs=[f"{kclvm_ROOT}/runtime/src"],
        )
    ],
)
