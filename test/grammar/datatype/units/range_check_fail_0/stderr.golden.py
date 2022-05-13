
import sys
import kclvm.kcl.error as kcl_error
import os

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(err_type=kcl_error.ErrType.IntOverflow_TYPE,
                            file_msgs=[
                                kcl_error.ErrFileMsg(
                                    filename=cwd + "/main.k",
                                    line_no=1
                                ),
                            ],
                            arg_msg=kcl_error.INT_OVER_FLOW_MSG.format(1329227995784915872903807060280344576, 64))
    , file=sys.stdout
)

