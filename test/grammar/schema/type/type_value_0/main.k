schema Person:
    name: str
    age: int

NamedPerson = Person
alice = NamedPerson {
    name = "Alice"
    age = 18
}
