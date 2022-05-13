schema Name:
    name: str

schema Person:
    firstName: str | Name
    lastName: str

JohnDoe = Person {
    "firstName": 2,
    "lastName": "Doe"
}
