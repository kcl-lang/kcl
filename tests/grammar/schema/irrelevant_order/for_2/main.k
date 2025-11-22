schema Data:
    keys = [k for k in data]
    data: {str:str} = {
        "key1": "value1"
    }
    data = {
        "key2": "value2"
    }
    dataOther = {**data}

data = Data {}
