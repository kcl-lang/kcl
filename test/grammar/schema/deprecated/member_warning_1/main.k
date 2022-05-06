schema Person:
    firstName?: str = "John"
    lastName?: str
    @deprecated(strict=False)
    name?: str

JohnDoe = Person {
    name: "deprecated"
}
