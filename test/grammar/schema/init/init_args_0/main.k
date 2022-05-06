schema Person[separator]:
    firstName: str = "John"
    lastName: str
    fullName: str = firstName + separator + lastName

JohnDoe = Person('_') {
    "lastName": "Doe"
}
