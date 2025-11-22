type UnionType = int | str | bool

schema Data[a: UnionType]:
    if typeof(a) == "int":
        b = (a as int) + 1
    elif typeof(a) == "str":
        c = (a as str) + "1"
    elif typeof(a) == "bool":             
        d = (a as bool)
    else:
        e = a

x0 = Data(0)
x1 = Data("s")
x2 = Data(True)
