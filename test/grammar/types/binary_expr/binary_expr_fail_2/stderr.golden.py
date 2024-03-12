import sys
import os

import kclvm.kcl.error as kcl_error

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(
        err_type=kcl_error.ErrType.TypeError_Compile_TYPE,
        file_msgs=[
            kcl_error.ErrFileMsg(
                filename=cwd + "/main.k",
                line_no=1,
                col_no=10,
            )
        ],
        arg_msg="unsupported operand type(s) for <: '{any:any}' and '{any:any}'"
    )
    , file=sys.stdout
)
