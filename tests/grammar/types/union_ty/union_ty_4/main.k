schema A:
    a: int

schema X:
    x: A | "1"

x = X {
    x.a = 1
}
