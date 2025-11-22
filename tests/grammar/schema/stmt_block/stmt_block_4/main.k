schema Person:
    firstName: str = "John"
    lastName: str
    name: str = "Doe"

    _name = "123"
    name += _name

JohnDoe = Person {
    "lastName": "Doe",
}
