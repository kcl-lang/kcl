sumFunc1 = lambda x, y {
    x + y
}
sumFunc2 = lambda x, y = 1 {
    x + y
}
sumFunc3 = lambda x = 1, y: int = 1 {
    x + y
}
sumFunc4 = lambda x: int = 1, y: int = 1 -> int {
    x + y
}
x0 = sumFunc1(1, 2)
x1 = sumFunc1(2, 3)
x2 = sumFunc1(3, 4)
x3 = sumFunc1(4, 5)

schema Data:
    var: int = 1
    _func = lambda x: int | str, y: int | str {
        (lambda x, y {
            int(x) + int(y) + var
        })(x, y)  
    }

    a = _func(1, 1)
    b = _func("123", "456")

data = Data()

result = (lambda x: int, y: int -> int {
    a = 1
    (lambda {
        x + y + a + 1
    })()
})(1, 1)
