
import sys
import kclvm.kcl.error as kcl_error
import os

cwd = os.path.dirname(os.path.realpath(__file__))

main_file = os.path.join(cwd, 'main.k')
module_file = os.path.join(cwd, 'module.k')

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(err_type=kcl_error.ErrType.CompileError_TYPE,
                            file_msgs=[
                                kcl_error.ErrFileMsg(
                                    filename=module_file,
                                    line_no=2,
                                    col_no=1,
                                ),
                            ],
                            arg_msg=f"Cannot import {main_file} in the main package")
    , file=sys.stdout
)

