schema Person:
    name: str = "Alice"

_person = Person {}
_person = _person | {"key": "value"}
person = _person
