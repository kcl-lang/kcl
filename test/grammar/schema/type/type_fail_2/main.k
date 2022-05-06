schema Name0:
    firstName: str
    lastName: str

schema Name1:
    firstName: str
    lastName: str

schema Person:
    name: Name0
    age: int

alice = Person {
    "name": Name1 {
        "firstName": "Alice",
        "lastName": "Terry"
    },
    "age": 12
}
