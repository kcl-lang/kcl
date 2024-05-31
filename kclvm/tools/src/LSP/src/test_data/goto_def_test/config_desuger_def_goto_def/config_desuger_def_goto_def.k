schema Person:
    n: Name

schema Name:
    name: str

p4 = Person {n.name: "a"}

