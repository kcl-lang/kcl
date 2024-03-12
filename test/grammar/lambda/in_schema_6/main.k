schema Name:
    y: int = 2
    z: int = lambda x {
        y = 1
        z = y + 1  # y is 1 instead of 2
    }(1)


n = Name {}
