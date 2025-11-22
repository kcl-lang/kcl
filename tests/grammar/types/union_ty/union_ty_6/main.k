schema A:
    a: int

schema B:
    b: int

schema X:
    x: A | "1"
    y: "2" | B

x = X {
    x.a = 1
    y.b = 2
}
