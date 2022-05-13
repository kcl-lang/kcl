schema Data:
    i: int = 0
    data: [int] = [1, 2, 3]
    dataNew: [int] = [i * 2 for i in data]

data = Data {}
