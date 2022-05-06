
import sys
import kclvm.kcl.error as kcl_error
import os

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(err_type=kcl_error.ErrType.TypeError_Compile_TYPE,
                            file_msgs=[
                                kcl_error.ErrFileMsg(
                                    filename=cwd + "/main.k",
                                    line_no=13,
                                    col_no=5,
                                    arg_msg="expect Name1",
                                    err_level=kcl_error.ErrLevel.ORDINARY
                                ),
                                kcl_error.ErrFileMsg(
                                    filename=cwd + "/main.k",
                                    line_no=38,
                                    col_no=21,
                                    arg_msg="got int(1)"
                                ),
                            ],
                            arg_msg="expect Name1, got int(1)")
    , file=sys.stdout
)

