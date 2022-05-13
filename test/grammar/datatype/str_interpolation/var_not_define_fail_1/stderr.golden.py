import os
import sys

import kclvm.kcl.error as kcl_error

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(
        err_type=kcl_error.ErrType.CompileError_TYPE,
        file_msgs=[
            kcl_error.ErrFileMsg(
                filename=cwd + "/main.k",
                line_no=2,
                col_no=8
            )
        ],
        arg_msg="name 'c' is not defined"
    ),
    file=sys.stdout
)

