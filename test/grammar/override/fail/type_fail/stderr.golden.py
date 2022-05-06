import sys
import kclvm.kcl.error as kcl_error
import os

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(
        err_type=kcl_error.ErrType.TypeError_Compile_TYPE,
        file_msgs=[
            kcl_error.ErrFileMsg(
                filename=cwd + "/main.k",
                line_no=4,
                col_no=5,
                arg_msg="expect int",
                err_level=kcl_error.ErrLevel.ORDINARY
            ),
        ],
        arg_msg="expect int, got str(A)"
    ),
    file=sys.stdout
)

