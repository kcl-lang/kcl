import sys
import os

import kclvm.kcl.error as kcl_error

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(err_type=kcl_error.ErrType.EvaluationError_TYPE,
                            file_msgs=[
                                kcl_error.ErrFileMsg(
                                    filename=cwd + "/main.k",
                                    line_no=2,
                                ),
                            ],
                            arg_msg="failed to update the dict. An iterable of key-value pairs was expected, but got UndefinedType. Check if the syntax for updating the dictionary with the attribute 'b' is correct")
    , file=sys.stdout
)
