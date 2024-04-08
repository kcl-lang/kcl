schema Config:
    a: int = b
    b: int = 2
    if True:
        b = 1
        b += 3

x0 = Config {}
x1 = Config {
    a = 10
}
