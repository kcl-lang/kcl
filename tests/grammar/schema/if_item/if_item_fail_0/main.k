schema Data:
    key1?: str
    key2?: str
    key3?: str

data1 = Data {"key1": "value1", "key2": "value2"}
data2 = Data {
    **data1
    if data1["key1"]: "key3": data1["key1"]
}
data3 = Data {
    **data2
    if data2["key3"] == "value3": "key4": "value4"
}
