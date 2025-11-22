import manifests

manifests.yaml_stream([{
    name = "Alice"
    age = 10
}, {
    name = "Bob"
    age = None
}], opts = {
    ignore_none = True
})
