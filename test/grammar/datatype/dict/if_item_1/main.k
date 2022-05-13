data1 = {"key1": "value1", "key2": "value2"}
data2 = {
    **data1
    if data1["key1"]: "key3": data1["key1"]
}
data3 = {
    **data2
    if data2["key3"] == "value3": "key4": "value4"
}
