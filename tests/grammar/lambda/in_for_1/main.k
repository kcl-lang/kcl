f = lambda x, y, z {
    x * y + z
}
schema Data:
    val: int = 0
    x0 = [
        (lambda x, y, z {
            x + y
        })(x, y, val) for x in [1, 2] for y in [1, 2]
    ]
    x1 = [f(x, y, val) for x in [1, 2] for y in [1, 2]]

data = Data {
    val = 1
}
