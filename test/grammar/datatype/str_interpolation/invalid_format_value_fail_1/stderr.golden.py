import os
import sys

import kclvm.kcl.error as kcl_error

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(
        err_type=kcl_error.ErrType.InvalidFormatSpec_TYPE,
        file_msgs=[
            kcl_error.ErrFileMsg(
                filename=cwd + "/main.k",
                line_no=3,
                col_no=8,
                end_col_no=37
            )
        ],
        arg_msg="invalid single '$', expecting '$' or '{'"
    ),
    file=sys.stdout
)

