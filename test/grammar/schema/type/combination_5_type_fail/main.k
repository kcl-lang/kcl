schema Name:
    firstName: int
    lastName: str

schema Person:
    name: Name
    age: int = 20

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
            "name": Name {
                "firstName": "Alice",
                "lastName": "Green"
            },
            "age": 10
        }
    ]
}
