global: int = 10

schema Data:
    var: int = 1
    _func = lambda x: int | str, y: int | str {
        (lambda x, y {
            int(x) + int(y) + var + global
        })(x, y)  
    }

    a = _func(1, 1)
    b = _func("123", "456")

data = Data()
