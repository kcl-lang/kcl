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
                line_no=1,
                col_no=1,
                arg_msg='expect {str:str}',
                err_level=kcl_error.ErrLevel.ORDINARY,
            ),
            kcl_error.ErrFileMsg(
                filename=cwd + "/main.k",
                line_no=2,
                col_no=1,
                arg_msg='got {str:int}',
            ),
        ],
        arg_msg='can not change type of _a',
    ),
    file=sys.stdout,
)
