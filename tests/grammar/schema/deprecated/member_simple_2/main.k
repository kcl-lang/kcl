schema Person:
    firstName?: str = "John"
    @deprecated
    lastName?: str = None
    name?: str = None

JohnDoe = Person {
    name: "deprecated"
}
