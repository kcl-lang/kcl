schema Person:
    firstName: str
    lastName: str

JohnDoe = Person {
    "firstName": "John",
    "lastName": "Doe",
    "fullName": "John Doe"  # undefined field
}
