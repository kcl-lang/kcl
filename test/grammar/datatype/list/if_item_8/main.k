data0: [int] = [
    if False:
        *[0]
    else:
        1
        2
]
data1: [int] = [
    if False:
        *[0]
    elif False:
        *[1]
    else:
        *[2]
]
data2: [int] = [
    if False:
        *[0]
    else:
        if False:
            *[1]
        else:
            *[2]
]
