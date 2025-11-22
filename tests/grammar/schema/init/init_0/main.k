schema Person:
    firstName: str = "John"
    lastName: str
    fullName: str = firstName + ' ' + lastName

JohnDoe = Person {
    "lastName": "Doe"
}
