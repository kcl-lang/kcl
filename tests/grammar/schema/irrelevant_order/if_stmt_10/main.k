schema Name:
    _a = 1
    _b = 1
    if True:
        _a += 1
        _b += 1
    elif False:
        _a += 1
        _b += 1
    a = _a
    b = _b

name = Name{}
