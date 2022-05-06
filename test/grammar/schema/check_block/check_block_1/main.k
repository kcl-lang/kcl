
schema Person:
    firstName: str = "John"
    lastName: str
    check:
        len(lastName) > 0
        len(firstName) > 0

JohnDoe = Person {
    "lastName": "Doe"
}
