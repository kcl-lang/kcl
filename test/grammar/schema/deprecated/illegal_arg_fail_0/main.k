schema Person:
    firstName: str = "John"
    @deprecated(strict=False, "1.16")
    lastName: str
    name: str

JohnDoe = Person {
    name: "deprecated"
}
