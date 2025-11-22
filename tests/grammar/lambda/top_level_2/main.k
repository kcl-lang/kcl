func = lambda x: int | str, y: int | str {
    (lambda x, y {
        int(x) + int(y)
    })(x, y)  
}
afunc = func
a = afunc(1, 1)
b = afunc("123", "456")
