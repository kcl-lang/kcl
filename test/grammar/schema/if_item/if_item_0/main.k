schema Data:
    key1?: str
    key2?: str
    key3?: str

data1 = Data {
    if False: key1: "value1"
    key2: "value2"
    if True: key3: "value3"
}
data2 = Data {
    key1: "value1"
    if data1["key2"]:
        key2: data1["key2"]
}
