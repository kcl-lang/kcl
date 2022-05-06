schema Data:
    shouldAddKey: bool = False
    data: {str:} = {
        if shouldAddKey: key1: "value1"
        key2: "value2"
    }

data = Data {}
