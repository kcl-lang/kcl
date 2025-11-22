schema Person:
    firstName: str = "John"
    lastName: str
    name: str = (firstName + " " + lastName) if len(lastName) > 0 else firstName

JohnDoe = Person {
    "lastName": "Doe"
}
