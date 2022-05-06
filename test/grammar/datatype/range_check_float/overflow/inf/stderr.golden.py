import sys
import kclvm.kcl.error as kcl_error
import os

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(
        err_type=kcl_error.ErrType.FloatOverflow_TYPE,
        file_msgs=[
            kcl_error.ErrFileMsg(
                filename=cwd + "/main.k",
                line_no=6
            )
        ],
        arg_msg=kcl_error.FLOAT_OVER_FLOW_MSG.format("inf", 64)
    )
    , file=sys.stdout
)

