schema Test:
    a: str
    b: str = "a={}".format(a)
    c: str

t = Test {
    a = "v1",
    b = "v2",
    c = "v3"
}
