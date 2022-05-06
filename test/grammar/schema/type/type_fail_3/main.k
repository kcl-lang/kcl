schema Name:
    firstName: str
    lastName: str

schema Person:
    name: Name
    age: int

schema User:
    person: Person
    phone: int

alice = User {
    "person": {
        "name": {
            "firstName": "Alice",
            "lastName": "Terry",
            "fullName": "Alice Terry"
        },
        "age": 12
    },
    "phone": 123
}
