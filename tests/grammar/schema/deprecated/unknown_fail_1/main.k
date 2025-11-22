@err_deprecated
schema Person:
    firstName: str = "John"
    lastName: str
    name: str

JohnDoe = Person {
    name: "deprecated"
}
