import sys
import os

import kclvm.kcl.error as kcl_error

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(err_type=kcl_error.ErrType.SchemaCheckFailure_TYPE,
                            file_msgs=[
                                kcl_error.ErrFileMsg(
                                    filename=cwd + "/main.k",
                                    line_no=9,
                                    arg_msg = "Check failed on the condition"
                                ),
                                kcl_error.ErrFileMsg(
                                    filename=cwd + "/main.k",
                                    line_no=16,
                                    col_no=1,
                                    arg_msg = "Instance check failed"
                                ),
                            ],
                            arg_msg="Check failed on check conditions")
    , file=sys.stdout
)

