schema LabelMap:
    [str]: str | int

schema NamedMap:
    [str]: {str:str} = {"default_key": "default_value"}

labelMap = LabelMap {
    "key1": "value1"
    "key2": 2
}

namedMap = NamedMap {
    "key1": {"key1": "value1"}
    "key2": {"key2": "value2"}
}
