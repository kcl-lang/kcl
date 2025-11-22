schema Data:
    key1?: str
    key2?: str

schema NamedData:
    shouldAddKey: bool = False
    data: Data = {
        if shouldAddKey:
            key1: "value1"
        else:
            key1: "1"
            key2: "2"
        key2: "value2"
    }

data = NamedData {
    if True:
        shouldAddKey = True
}
