import import_test.a # UnusedImport
import import_test.a # ReImport
import regex

schema Person:
    name: str
    age: int

b1 = b._b

import import_test.b  # ImportPosition

requires = option("params").requires or []
# Define the validation function
validate_required_labels = lambda item, requires {
    if requires:
        requires_map = {r.key: r.allowedRegex or "" for r in requires}
        labels = item.metadata.labels or {}
        if labels:
            assert all k, v in labels {
                regex.match(v, requires_map[k]) if requires_map[k]
            }, "must provide labels with the regex ${requires_map}"
    item
}
# Validate All resource
items = [validate_required_labels(i, requires) for i in option("items")]
