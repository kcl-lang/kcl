import import_file
# test for builtin
import math
# test for plugin
import kcl_plugin.app_context as ctx

schema FullName:
    firstName: str
    lastName: str

schema Person1(Person):
    name: FullName = FullName{
        firstName = "hello"
        lastName = "world"
    }
    card: str = "abc"
    _mybkLabels?: {str:str} = {
        if _workspace: "app.stack.io/workspace": _workspace.lower()
        "app.kubernetes.io/name": _appName
        "app.kubernetes.io/instance": _appName
    } if _globalTenantName == "CLOUDCORE" else Undefined
    commands: [str] = ["a", if True : "b"]

schema Person2(Person1):
    card: str
