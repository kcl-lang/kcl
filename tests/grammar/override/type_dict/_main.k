schema Name:
    firstName: str
    lastName: str

schema Person:
    name: Name
    age: int

schema Group:
    persons: {str:Person}

group = Group {
    "persons": {
        "me": {
            "name": {
                "firstName": "Alice",
                "lastName": "Terry"
            },
            "age": 12
        }
    }
}
