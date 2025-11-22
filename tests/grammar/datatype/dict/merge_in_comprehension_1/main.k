x = {
    a = 1
    b = True
}
y = {
    c = 3
    d = "4"
}
z1 = {**x, **y}
z2 = {"foo": {**i, a = 2, e = "five"} for i in [x, y]}.foo
