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
                col_no=19,
                end_col_no=26
            )
        ],
        arg_msg="the number of loop variables is 3, which can only be 1 or 2"
    ),
    file=sys.stdout
)
