schema A:
    a: int

schema B(A):
    b: int = a

schema X:
    x: B

x = X {
    x.a = 1
}
