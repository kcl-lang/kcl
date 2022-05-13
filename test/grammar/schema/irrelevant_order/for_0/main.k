schema Data:
    dataTrue: [int] = [k * e for k in data] if e else Undefined
    data = [1, 2, 3]
    e = 2

data = Data {}
