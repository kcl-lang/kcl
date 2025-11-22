schema A(A):
    name: str

schema B(C):
    name: str

schema C(B):
    name: str


schema D:
    mixin [D]

mixin D for D