schema Person:
    firstName: str = "John"
    lastName: str
    list: [str] = []

JoeDoe = Person {
    "firstName" = "Joe",
    "lastName" = "Doe",
    "list" = ["value"]
}
