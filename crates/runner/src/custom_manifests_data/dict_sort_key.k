import manifests

manifests.yaml_stream([{
    name = "kcl"
    age = 10
}, {
    age = 10
    name = "kcl"
}], opts = {sort_keys = True})
