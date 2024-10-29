import manifests

schema XYZ:
    [str]: str

xyz = {
    "_foo": "bar"
    "foo": "bar"
    xyz: XYZ {
        _bla: "bla"
    }
    a = None
}

manifests.yaml_stream([xyz], opts={
    ignore_private = True
})
