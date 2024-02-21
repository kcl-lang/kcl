import sys
import os

import kclvm.kcl.error as kcl_error

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(kcl_error.get_exception(err_type=kcl_error.ErrType.EvaluationError_TYPE,
                                                          file_msgs=[
                                                              kcl_error.ErrFileMsg(
                                                                  filename=cwd + "/main.k",
                                                                  line_no=3
                                                                  column_no=1,
                                                              )],
                                                          arg_msg="failed to access the file 'not_exist.txt':No such file or directory (os error 2)"
                                                          ),
                                  file=sys.stdout)
