@deprecated(strict=False, version="1.16", reason="use SuperPerson instead")
schema Person:
    firstName?: str = "John"
    lastName?: str
    name?: str

JohnDoe = Person {
    name: "deprecated"
}
