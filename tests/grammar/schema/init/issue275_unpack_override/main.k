schema MyXRD:
    type: str = "MyXRD"
    name: str = "Unknown"
    sizes?: [str] = ["Small", "Medium"]

# Crossplane XR data is typically a dict literal that uses the `:`
# (Union) operator for each entry; when the user unpacks it with `**`
# into a schema call, the entries should be treated as override-style
# assignments rather than per-key unions. Without the fix, the
# list-valued attribute was merged position-wise with the schema default
# (`sizes = ["Small", "Medium"]` ∪ `["Large"]` → `["Large", "Medium"]`),
# which is unintuitive and breaks the function-kcl use case
# (crossplane-contrib/function-kcl#275 / kcl-lang/kcl#275).
input = {
    name: "test-oxr"
    sizes: ["Large"]
}

oxr = MyXRD{**input}
assert oxr.sizes == ["Large"], "oxr.sizes should be ['Large'] (got ${oxr.sizes})"