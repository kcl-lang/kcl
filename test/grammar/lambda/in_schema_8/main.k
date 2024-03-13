x: int = 1
schema Name:
    x: int = 2
    z: int = lambda a {
        a + x # x is the schema x instead of global x
    }(2)

n = Name {}
