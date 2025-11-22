schema Data[arg1: int, arg2: int = 2]:
    id1: int = arg1
    id2: int = arg2

data0 = Data(1, 2)
data1 = Data(arg1=1, arg2=2)
data2 = Data(1, arg2 = 2)
data3 = Data(arg2=2, arg1=1)
data4 = Data(1)
