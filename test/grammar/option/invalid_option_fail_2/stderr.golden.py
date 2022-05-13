import sys
import kclvm.kcl.error as kcl_error

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(
        err_type=kcl_error.ErrType.IllegalArgumentError_TYPE,
        arg_msg="Invalid value for option \"--argument(-D)\": Invalid option name: ''. should be a non-empty string"
    ),
    file=sys.stdout
)

