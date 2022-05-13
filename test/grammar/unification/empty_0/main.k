schema Person:
    name?: str = None
    age?: int = None

alice = Person {}
alice = Person {}

name = alice.name or "Alice"
age = alice.age or 18
