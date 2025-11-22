schema Person:
    firstName: str = "John"
    lastName: str

JohnDoe = Person {
    "lastName": "Doe"
}

JohnDoe.lastName = "John0"

