schema Name:
    firstName?: str
    lastName?: str

schema Person:
    name: Name
    name: Name |= {
        "lastName": "Dow"
    }
    gender: str = "default gender"

alice = Person {
    "name": {
        "firstName": "John"
    },
    "gender" = "female"
}
