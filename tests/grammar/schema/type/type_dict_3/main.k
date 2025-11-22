schema Name:
    firstName: str
    lastName: str

schema Person:
    name: Name
    age: int

schema Group:
    persons: [Person]

group = Group {
    "persons": [{
        "name": {
            "firstName": "Alice",
            "lastName": "Terry"
        },
        "age": 12
    }]
}
