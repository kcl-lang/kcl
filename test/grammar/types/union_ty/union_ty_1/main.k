schema A:
    a: int
    
schema B:
    b: int
    
schema X:
    x: A | B

x = X {
    x.b = 1
}
