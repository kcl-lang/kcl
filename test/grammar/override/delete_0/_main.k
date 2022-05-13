schema Person:
    firstName?: str
    lastName?: str
    fullName: str = (firstName or "") + "_" + (lastName or "")

JohnDoe = Person {
    firstName: "John"
    lastName: "Doe"
}
