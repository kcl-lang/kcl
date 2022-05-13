
myDict = {"key1": "value1", "key2": "value2", "key3": "value3"}

result = {k: myDict[k] for k in myDict if k in ["key1", "key2"]}
