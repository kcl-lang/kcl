a = 1  # 1
b = 1  # 1
data1 = [
    *[1]
    if a == 1:
        *[2, 3 ,4],
        if b == 1:
            b
        else:
            b + 1
    elif a == 2:
        if b == 1:
            b
    else:
        b
]  # [1, 2, 3, 4, 1]
data2 = [
    if a == 1: if b == 1: a + b
]  # [1, 2]
