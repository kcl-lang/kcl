schema Data:
    key1?: str
    key2?: str

schema NamedData:
    data: Data = Data {
        if True: key1: "value1"
        key2: "value2"
    }

data = NamedData {}
