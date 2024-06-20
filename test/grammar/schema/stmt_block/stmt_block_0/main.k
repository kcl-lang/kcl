schema Person:
    firstName: str = "John"
    lastName: str

    if len(lastName) > 0:
        _name = firstName + " " + lastName
    else:
        _name = firstName

    name: str = _name

JohnDoe = Person {
    "lastName": "Doe"
}
