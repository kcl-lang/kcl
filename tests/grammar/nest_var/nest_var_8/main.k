schema Person:
    labels: {str:}

base = {"labels": {"key": "value"}}

person = Person {
    **base
    labels.key = "override"
}
