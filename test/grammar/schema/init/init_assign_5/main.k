_dict = {"k": "v", "key": "value"}

schema Person:
    labels: {str:str}

alice = Person {
    "labels": {k: _dict[k] for k in _dict if k in ["key"]}
}
