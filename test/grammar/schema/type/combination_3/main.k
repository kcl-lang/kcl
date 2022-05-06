schema Name:
    firstName: str
    lastName: str

schema Person:
    name: Name
    age: int

schema Rule:
    entity: Person
    access: [str]

schema Model:
    access: [str]
    user: {str:}
    rule: Rule = {
        "access": access,
        "entity": user
    }

alice = Model {
    "user": {
        "name": {
            "firstName": "alice",
            "lastName": "Green"
        },
        "age": 10
    },
    "access": ["read", "write"]
}
