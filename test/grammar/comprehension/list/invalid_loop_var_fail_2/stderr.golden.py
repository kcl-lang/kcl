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
                end_col_no=22
            )
        ],
        arg_msg="loop variables can only be ordinary identifiers"
    ),
    file=sys.stdout
)

