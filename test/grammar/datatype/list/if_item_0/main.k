data1 = [1, if True: 2, 3]
data2 = [
    1
    if False: 2
    3
]
data3 = [
    if data2[0] == 1: data2[0]
    if data1[1] == 2:
        data1[1]
]
