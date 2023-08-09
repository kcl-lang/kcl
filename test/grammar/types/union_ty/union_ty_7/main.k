schema A:
    a: int

schema B:
    b: int

schema X:
    x: {str: A | B}

x = X {
    x.a.a = 1
    x.b.b = 2
}
