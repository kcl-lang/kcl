schema A:
    a: int

schema B(A):
    b: int = a

schema X:
    x: A | B

x = X {
    x.a = 1
    x.b = 2
}
