data0 = (lambda x, y {
    z = x * 2
    z + y
})(1, 1)

schema Data:
    data = [(lambda x, y {
        _z = 2 * x
        _z = _z + y
    })(x, y) for x in [1, 2] for y in [1, 2]]

data = Data()
