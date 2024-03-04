import sys
import kclvm.kcl.error as kcl_error
import os

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(
        err_type=kcl_error.ErrType.CompileError_TYPE,
        file_msgs=[
            kcl_error.ErrFileMsg(
                filename=cwd + "/main.k",
                line_no=3,
                col_no=6,
            )
        ],
        arg_msg="'else if' here is invalid in KCL, consider using the 'elif' keyword",
        file=sys.stdout,
    )
)
