schema Name:
    firstName: str
    lastName: str

schema Person:
    name: Name
    age: int

schema Model:
    name: {str:str}
    age: int
    personModel: Person = {
        "name": name,
        "age": age
    }

alice = Model {
    "name": {
        "firstName": "alice",
        "lastName": "Green"
    },
    "age": 10
}
