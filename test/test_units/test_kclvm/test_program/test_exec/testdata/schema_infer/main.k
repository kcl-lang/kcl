import pkg

schema Person:
    name: str = "kcl"
    age: int = 1

x0: pkg.Person = {
    age = 101
}

x1: Person = {
    age = 101
}
x2: Person = Person()
