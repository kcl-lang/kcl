schema Data:
    key1?: str
    key2?: str

schema NamedData:
    shouldAddKey: bool = False
    data: Data = {
        if shouldAddKey: key1: "value1"
        key2: "value2"
    }

data = NamedData {}
