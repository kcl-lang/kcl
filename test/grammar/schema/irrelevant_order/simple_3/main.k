schema Data0:
    value: int
    if n == 0:
        value = 0
    n1: int = n + 1
    n: int = 0

schema Data1:
    value: int
    if n == 0:
        value = 0
    n: int = 0
    n1: int = n + 1

data0 = Data0 {}
data1 = Data1 {}
