import sys
import os

import kclvm.kcl.error as kcl_error

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(err_type=kcl_error.ErrType.CannotAddMembers_TYPE,
                            file_msgs=[
                                kcl_error.ErrFileMsg(
                                    filename=cwd + "/main.k",
                                    line_no=13,
                                    col_no=35,
                                    arg_msg="'key4' is not defined in schema 'Data'"
                                ),
                            ],
                            arg_msg="Cannot add member 'key4' to schema 'Data'")
    , file=sys.stdout
)

