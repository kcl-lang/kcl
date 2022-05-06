import sys
import kclvm.kcl.error as kcl_error
import os

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(err_type=kcl_error.ErrType.TypeError_Compile_TYPE,
                            file_msgs=[
                                kcl_error.ErrFileMsg(
                                    filename=cwd + "/pkg/person.k",
                                    line_no=3,
                                    col_no=5,
                                    arg_msg="expect [pkg.Container]",
                                    err_level=kcl_error.ErrLevel.ORDINARY
                                ),
                                kcl_error.ErrFileMsg(
                                    filename=cwd + "/main.k",
                                    line_no=5,
                                    col_no=9,
                                    arg_msg="got {str(image)|str(name):str(image)|str(name)}"
                                ),
                            ],
                            arg_msg="expect [pkg.Container], got {str(image)|str(name):str(image)|str(name)}")
    , file=sys.stdout
)

