schema D[name: str]: # ''
    n: str = name

d = D() { # expected 1 positional argument, found 0
    n = ""
}