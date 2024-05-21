schema Person:
    firstName?: str = "John"
    lastName?: str
    @deprecated(strict=True)
    name?: str

JohnDoe = Person {
    name: "deprecated"
}
