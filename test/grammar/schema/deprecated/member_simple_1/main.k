schema Person:
    firstName: str = "John"
    @deprecated
    lastName: str
    @deprecated
    name: str

JohnDoe = Person {
    name: "deprecated"
}
