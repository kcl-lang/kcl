schema Person:
    name: str = "Alice"
    age: int = 18

Base = 1

schema PersonMixin:
    nameUpper: str = name.upper()
