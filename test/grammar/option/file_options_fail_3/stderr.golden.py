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
        arg_msg="""Invalid yaml content of setting file:
while scanning a quoted scalar
  in "<unicode string>", line 1, column 1:
    "
    ^ (line: 1)
found unexpected end of stream
  in "<unicode string>", line 1, column 2:
    "
     ^ (line: 1)"""
    ),
    file=sys.stdout
)
