
import sys
import kclvm.kcl.error as kcl_error
import os

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(err_type=kcl_error.ErrType.Deprecated_TYPE,
                            file_msgs=[
                                kcl_error.ErrFileMsg(
                                    filename=cwd + "/main.k",
                                    line_no=10,
                                ),
                            ],
                            arg_msg=kcl_error.DEPRECATED_WARNING_MSG.format("Person", ""))
    , file=sys.stdout
)

