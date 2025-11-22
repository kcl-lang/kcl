params = option("params") or {}
# Use `k = v` to override existing annotations
annotations: {str:str} = {k = v for k, v in params.annotations or {}}
items = [item | {
    metadata.annotations: annotations
} for item in option("items")]