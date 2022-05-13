schema Person:
    name: str
    age: int

alice = Person {
    if True:
        name: "Alice"
}

alice = Person {
    if True:
        age = 18
}

alice = Person {
    age = 10
}

name = alice.name
age = alice.age
