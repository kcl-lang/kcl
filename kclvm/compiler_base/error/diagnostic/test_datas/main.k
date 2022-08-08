schema Person:
    firstName: str = "John"
    lastName: str
    name: str

    _name: int = 123
    name = _name

JohnDoe = Person {
    "lastName": "Doe"
}
