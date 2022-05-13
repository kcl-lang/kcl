import sys
import kclvm.kcl.error as kcl_error
import os

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_warning_message(
    kcl_error.get_exception(
        err_type=kcl_error.ErrType.FloatUnderflow_TYPE,
        file_msgs=[
            kcl_error.ErrFileMsg(
                filename=cwd + "/main.k",
                line_no=7
            )
        ],
        arg_msg=kcl_error.FLOAT_UNDER_FLOW_MSG.format(1.1754943509999997e-38, 32)
    )
    , file=sys.stdout
)
