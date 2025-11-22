
schema Person:
    firstName: str = "John"
    lastName: str
    check:
        len(lastName) > 0

JohnDoe = Person {
    "lastName": "Doe"
}
