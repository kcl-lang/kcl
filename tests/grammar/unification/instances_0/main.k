schema Person:
    name: str
    age: int

_person = Person {name: "Alice", age: 18}
_persons = Person.instances()
persons = _persons
