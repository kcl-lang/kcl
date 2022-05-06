schema Test:
    a: str
    b: str = "a={}".format(a)
    c: str

t = Test {
    a: "v1",
    c: "v2"
}
