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

name = alice.name
age = alice.age
