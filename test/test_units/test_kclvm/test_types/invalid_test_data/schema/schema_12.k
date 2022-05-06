schema NamedMap:
    [str]: str

data = {
    key1: "value1"
    key2: "value2"
}

named_data = NamedMap {
    **data
    data: 1
}
