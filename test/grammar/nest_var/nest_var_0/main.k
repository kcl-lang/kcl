schema Son:
    metadata: {str:} | str
    name: str

_keyVals = {
    "key1": "value"
    "key2": 2
}

son = Son {
    metadata.labels.key1: 123,
    metadata.labels.key2: _keyVals,
    name: "name"
}
