import sys
import kclvm.kcl.error as kcl_error
import os

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(kcl_error.get_exception(err_type=kcl_error.ErrType.InvalidSyntax_TYPE,
                                                          file_msgs=[
                                                              kcl_error.ErrFileMsg(
                                                                  filename=cwd + "/main.k",
                                                                  line_no=1,
                                                                  col_no=4,
                                                                  arg_msg="Expected one of ['all', 'any', "
                                                                          "'bin_number', 'dec_number', 'False', "
                                                                          "'filter', 'float_number', 'hex_number', 'lambda', "
                                                                          "'{', '[', '(', 'long_string', 'not', 'map', "
                                                                          "'-', 'name', 'None', '~', 'oct_number', '+"
                                                                          "', 'string', 'True', 'Undefined']",
                                                              )],
                                                          ),
                                  file=sys.stdout)
