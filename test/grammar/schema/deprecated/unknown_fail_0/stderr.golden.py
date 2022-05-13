import sys
import kclvm.kcl.error as kcl_error
import os

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(
        err_type=kcl_error.ErrType.UnKnownDecorator_TYPE,
        file_msgs=[
            kcl_error.ErrFileMsg(
                filename=cwd + "/main.k",
                line_no=4,
                col_no=6
            )
        ],
        arg_msg=kcl_error.UNKNOWN_DECORATOR_MSG.format("err_deprecated")
    ),
    file=sys.stdout
)

