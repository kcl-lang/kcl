schema Person:
    fullName = firstName + ' ' + lastName
    firstName: str = "John"
    lastName?: str = None

JohnDoe = Person {
    "lastName": "Doe"
}
