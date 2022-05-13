func = lambda x: int | str, y: int | str {
    (lambda x, y {
        int(x) + int(y)
    })(x, y)  
}
a = func(1, 1)
b = func("123", "456")
