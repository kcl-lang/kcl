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
                line_no=6,
                col_no=18,
                arg_msg="got str(Golang)"
            )
        ],
        arg_msg="expect str(KCL)|str(CUE), got str(Golang)"
    ),
    file=sys.stdout
)

