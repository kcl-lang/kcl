schema Person:
    firstName: str = "John"
    lastName: str
    @deprecated
    name: str

JohnDoe = Person {
    lastName = "Doe"
    name: "deprecated"
}
