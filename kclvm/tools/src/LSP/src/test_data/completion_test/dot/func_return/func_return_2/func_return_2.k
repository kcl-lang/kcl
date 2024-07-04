schema Name:
    name: str

f1: (str) -> Name = lambda s: str -> Name {
    Name {name: s}
}

b = f1("a")
