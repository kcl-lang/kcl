var = {"key": "val"}
var1 = {
    "key1": "val1"
    "key2": "val2"
}

schema Person[varArg]:
    labels: {str:} = var
    annotations: {str:} = varArg

person = Person(var1) {}
