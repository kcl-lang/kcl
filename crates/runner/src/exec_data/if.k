a = 1 if True else 2
b = 2 if False else 3
c = 4 if a else b
ok = 3 if False else (2 if True else 4)

_a = 1
if True:
    if True:
        _a = 2
    elif True:
        _a = 3
    else:
        _a = 4
else:
    _a = 5

schema Data:
    id?: int = 1
    if True:
        if True:
            id = 2
        elif True:
            id = 3
        else:
            id = 4
    else:
        id = 5
