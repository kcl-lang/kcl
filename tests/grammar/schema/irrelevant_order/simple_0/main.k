schema Person:
    fullName: str = firstName + ' ' + lastName
    firstName: str = "John"
    lastName: str

JohnDoe = Person {
    lastName: "Doe"
}
