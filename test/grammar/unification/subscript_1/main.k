schema Person:
    name: str
    age: int

alice = Person {
    name: "Alice"
    age: 18
}

alice = Person {}

name = alice.name
age = alice.age
