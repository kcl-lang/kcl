import ..pkg

p = pkg.Person {
    name: "alice"
    age: 1
}

schema Name:
    name: str

schema Person:
    n: Name

p2 = Person {
    n: Name {name: pkg.m.name}
}

s = p2.n.name
