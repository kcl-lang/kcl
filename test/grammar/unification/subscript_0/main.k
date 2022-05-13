schema Person:
    name: str
    age: int

alice = Person {
    name: "Alice"
}

alice = Person {
    age: 18
}

name = alice.name
age = alice.age
