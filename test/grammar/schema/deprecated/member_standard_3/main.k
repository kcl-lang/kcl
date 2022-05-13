schema Person:
    firstName: str = "John"
    lastName: str = "Doe"
    @deprecated(version="1.16", reason="use firstName and lastName instead", strict=True)
    name: str

    name = firstName + lastName

JohnDoe = Person {
    name = "JohnDoe"
}
