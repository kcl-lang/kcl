schema Person:
    name: str = "Alice"

_personA = Person {}
_personA = _personA | {"name" = 123.0}
personA = _personA
