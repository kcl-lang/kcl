a = 1  # 1
data1 = [
    *[1]
    if a == 1:
        *[2]
    elif a == 2:
        *[3]
    else:
        *[3]
]  # [1, 2]
data2 = [
    *[1]
    if a == 1: *[2]
    elif a == 2: *[3]
    else: *[3]
]  # [1, 2]
