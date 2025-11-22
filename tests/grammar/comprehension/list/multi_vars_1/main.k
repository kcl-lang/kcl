data = [1, 2, 3]
dataLoop1 = [i + v for i, v in data]  # [1, 3, 5]
dataLoop2 = [v for i, v in data if v == 2]  # [2]
dataLoop3 = [v if v > 2 else v + i for i, v in data]  # [1, 3, 3]
