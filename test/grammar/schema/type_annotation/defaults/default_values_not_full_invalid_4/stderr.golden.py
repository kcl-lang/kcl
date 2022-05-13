import sys
import os

import kclvm.kcl.error as kcl_error

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(kcl_error.get_exception(err_type=kcl_error.ErrType.IllegalArgumentError_Syntax_TYPE,
                                                          file_msgs=[
                                                              kcl_error.ErrFileMsg(
                                                                  filename=cwd + "/main.k",
                                                                  line_no=1,
                                                                  col_no=20,
                                                                  end_col_no=28,
                                                                  arg_msg="A default argument"
                                                              )],
                                                          arg_msg="non-default argument follows default argument"
                                                          ),
                                  file=sys.stdout)
