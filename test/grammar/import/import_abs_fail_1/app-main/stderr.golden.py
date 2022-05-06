import sys
import kclvm.kcl.error as kcl_error
import os

cwd = os.path.dirname(os.path.realpath(__file__))
modulename = './../'

packagename = os.path.abspath(os.path.join(cwd, modulename))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(
        err_type=kcl_error.ErrType.CannotFindModule_TYPE,
        file_msgs=[
            kcl_error.ErrFileMsg(
                filename=cwd + "/main.k",
                line_no=1,
                col_no=1,
                end_col_no=31
            )
        ],
        arg_msg=kcl_error.CANNOT_FIND_MODULE_MSG.format("...some0.pkg1", packagename)
    ),
    file=sys.stdout
)

