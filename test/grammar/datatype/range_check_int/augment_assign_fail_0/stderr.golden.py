
import sys
import kclvm.kcl.error as kcl_error
import os

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(err_type=kcl_error.ErrType.IntOverflow_TYPE,
                            file_msgs=[
                                kcl_error.ErrFileMsg(
                                    filename=cwd + "/main.k",
                                    line_no=2
                                ),
                            ],
                            arg_msg=kcl_error.INT_OVER_FLOW_MSG.format(2147483648, 32))
    , file=sys.stdout
)

