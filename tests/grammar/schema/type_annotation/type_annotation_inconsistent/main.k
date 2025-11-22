schema A[a1, a2: int, a3: int]:
    a: int = a1
    b: int = a2
    c: str = a3


a = A(a1=1,a2=2,a3="3")