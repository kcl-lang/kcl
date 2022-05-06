schema Config:
    a: int = b
    b: int
    if True:
        b = 1
        b += 1

x0 = Config {}
x1 = Config {
    a = 10
}
