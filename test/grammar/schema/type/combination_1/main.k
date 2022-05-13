schema Name:
    firstName: str
    lastName: str

schema Person:
    name: Name
    age: int

schema Group:
    name: str
    persons: [Person]

schema Model:
    name: str
    persons: [{str:}]
    groupModel: Group = {
        "name": name,
        "persons": persons
    }

alice = Model {
    "name": "myModel",
    "persons": [
        {
            "name": {
                "firstName": "Alice",
                "lastName": "Green"
            },
            "age": 10
        }
    ]
}
