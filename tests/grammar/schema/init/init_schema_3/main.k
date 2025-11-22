schema Person:
    name: Name = {"first": "alice"}
    age: int

schema Name:
    first: str

alice = Person {
    "age": 10
}
