@deprecated(version="1.16", reason="use SuperPerson instead", strict=True)
schema Person:
    firstName: str = "John"
    lastName: str
    name: str

JohnDoe = Person {
    name: "deprecated"
}
