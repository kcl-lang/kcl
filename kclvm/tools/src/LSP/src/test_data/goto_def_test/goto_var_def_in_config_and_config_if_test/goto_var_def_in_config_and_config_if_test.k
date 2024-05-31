params = option("params")
toMatch = params.toMatch
toAdd = params.toAdd
items = [item | {
    # If all annotations are matched, patch more annotations
    if all key, value in toMatch {
        item.metadata.annotations[key] == value
    }:
        metadata.annotations: toAdd
    
} for item in option("items")]
