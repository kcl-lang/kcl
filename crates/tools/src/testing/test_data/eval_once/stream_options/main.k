import manifests

# Planning unifies this config expression with the global `_x` only when
# it shows hidden values, which this output doesn't.
{_x = 1}
_x = 2
manifests.yaml_stream([{}], opts = {ignore_private = True})
