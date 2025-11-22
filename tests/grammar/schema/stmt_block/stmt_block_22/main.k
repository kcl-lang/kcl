schema Data:
    name: str = "dataName"
    _data: [int] = []
    _hc: [int] = []
    if _hc and len(_hc) > 0:
        if len(_hc) > 3:
            temp1 = _hc
        elif len(_hc) > 1:
            temp2 = _hc
        else:
            temp3 = _hc

data = Data {}
