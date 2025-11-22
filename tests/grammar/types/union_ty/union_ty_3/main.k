schema A:
    a: int

schema B:
    b: int
    
schema X:
    x: [A | B]

x = X {
    x = [{a = 1}, {b = 2}]
}
