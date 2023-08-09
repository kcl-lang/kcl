schema A:
    a: int
    
schema B:
    b: int
    
schema C(B):
    c: int

schema X:
    x: A | B | C

x = X {
    x.d = 1
}
