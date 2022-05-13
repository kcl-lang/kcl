import sys
import kclvm.kcl.error as kcl_error

file = 'temp.yaml'

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(
        err_type=kcl_error.ErrType.IllegalArgumentError_TYPE,
        file_msgs=[
            kcl_error.ErrFileMsg(
                filename=file,
            )
        ],
        arg_msg="""Invalid configuration in setting file:
setting file content should be a mapping, got: :2"""
    ),
    file=sys.stdout
)
