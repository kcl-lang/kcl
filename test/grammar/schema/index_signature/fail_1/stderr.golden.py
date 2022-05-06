
import sys
import kclvm.kcl.error as kcl_error
import os

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(
        err_type=kcl_error.ErrType.IndexSignatureError_TYPE,
        file_msgs=[
            kcl_error.ErrFileMsg(
                filename=cwd + "/main.k",
                line_no=3,
                col_no=5,
                end_col_no=15
            )
        ],
        arg_msg="only one index signature is allowed in the schema"
    ),
    file=sys.stdout
)
