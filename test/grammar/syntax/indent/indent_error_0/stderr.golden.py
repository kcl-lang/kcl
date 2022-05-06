import sys
import kclvm.kcl.error as kcl_error
import os

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(err_type=kcl_error.ErrType.IndentationError_TYPE,
                            file_msgs=[
                                kcl_error.ErrFileMsg(
                                    filename=cwd + "/main.k",
                                    line_no=3,
                                    col_no=4,
                                )
                            ],
                            arg_msg=kcl_error.INDENTATION_ERROR_MSG.format("3")),
    file=sys.stdout
)
