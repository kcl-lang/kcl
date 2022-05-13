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
        "1": {
            "name": {
                "firstName": "Alice",
                "lastName": "Terry"
            },
            "age": 12
        }
    }
}
