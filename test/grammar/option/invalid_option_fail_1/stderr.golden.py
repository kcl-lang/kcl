
import sys
import kclvm.kcl.error as kcl_error
import os

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(
        err_type=kcl_error.ErrType.IllegalArgumentError_TYPE,
        arg_msg="Invalid value for option \"--argument(-D)\": should be in <name>=<value> pattern, got: key="
    ),
    file=sys.stdout
)

