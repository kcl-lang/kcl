schema Person:
    firstName: str = "John"
    lastName: str
    name: str = _name

    if len(lastName) > 0:
        _name = firstName + " " + lastName
    else:
        _name = firstName

JohnDoe = Person {
    "lastName": "Doe"
}
