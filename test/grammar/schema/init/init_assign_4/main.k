_dict = {"k": "v", "key": "value"}
_labels = {k: _dict[k] for k in _dict if k in ["key"]}

schema Person:
    labels: {str:str}

alice = Person {
    "labels": _labels
}
