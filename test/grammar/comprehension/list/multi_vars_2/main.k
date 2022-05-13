data = [1, 2, 3]
dataLoop1 = [i for i, _ in data]  # [0, 1, 2]
dataLoop2 = [v for _, v in data if v == 2]  # [2]
