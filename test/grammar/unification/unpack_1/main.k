schema Person:
    name: str
    age: int

_base = {
    name = "Bob"
    age = 10
}

alice = Person {
    age: 18
}

alice = Person {
    **_base
}

name = alice.name
age = alice.age
