schema Person:
    firstName: str = "John"
    lastName: str
    @deprecated(version="1.16", reason="use firstName and lastName instead", strict=True)
    name: str = "default_deprecated"

JohnDoe = Person {
    name = "override_deprecated"
}
