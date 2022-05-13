import sys
import kclvm.kcl.error as kcl_error
import os

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_warning_message(
    kcl_error.get_exception(
        err_type=kcl_error.ErrType.Deprecated_Warning_TYPE,
        arg_msg=kcl_error.DEPRECATED_WARNING_MSG.format("Person", "since version 1.16, use SuperPerson instead")
    )
    , file=sys.stdout
)

