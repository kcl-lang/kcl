schema Person:
    name: str
    age: int

_base = {
    name = "Bob"
    age = 18
}

alice = Person {
    **_base
}

alice = Person {
    name = "Alice"
    age = 18
}

name = alice.name
age = alice.age
