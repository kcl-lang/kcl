
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
                line_no=3,
                col_no=5,
                arg_msg="expect int",
                err_level=kcl_error.ErrLevel.ORDINARY
            ),
            kcl_error.ErrFileMsg(
                filename=cwd + "/main.k",
                line_no=10,
                col_no=9,
                arg_msg="got str(aa)"
            )
        ],
        arg_msg="expect int, got str(aa)"
    ),
    file=sys.stdout
)
