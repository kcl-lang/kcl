schema Config:
    a: int = b
    b: int = 1
    if True:
        b += 2

x0 = Config {}
x1 = Config {
    a = 10
}
