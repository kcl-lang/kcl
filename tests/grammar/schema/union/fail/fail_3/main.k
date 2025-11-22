schema Person:
    name: str = "Alice"

personA = Person {}
personB: Person = personA | {"name" = 123}
