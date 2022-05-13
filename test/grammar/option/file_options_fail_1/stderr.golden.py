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
invalid kcl_options value, should be list of key/value mapping. 
=== A good example will be:===
kcl_options:
  - key: myArg # the option key must be a string value
    value: myArgValue
=== got: ===
kcl_options:
   key: key
   value: value
"""
    ),
    file=sys.stdout
)
