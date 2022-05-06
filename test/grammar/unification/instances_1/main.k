schema Person:
    name: str
    age: int

_persons = [
    Person {name: "Alice", age: 18}
    Person {name: "Bob", age: 10}
]
_persons = Person.instances()
persons = _persons
