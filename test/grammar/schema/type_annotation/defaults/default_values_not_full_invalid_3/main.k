schema A[a1 = 100, a2, a3]:
    a: int = a1
    b: int = a2
    c: str = a3


a = A(a2=2,a3="3")