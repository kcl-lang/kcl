schema Person:
    firstName: str
    lastName: str
    fullName: str = firstName + "_" + lastName

JohnDoe = Person {
    firstName: "John"
    lastName: "Doe"
}
