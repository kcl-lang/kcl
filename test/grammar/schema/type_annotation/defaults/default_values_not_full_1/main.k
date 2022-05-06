schema A[a1, a2, a3="300"]:
    a: int = a1
    b: int = a2
    c: str = a3


a = A(a1 = 1,a2 = 2)