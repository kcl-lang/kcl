schema Person:
    firstName: str = "John"
    lastName: str
    age: int = 10

JohnDoe = Person {
    lastName: "Doe"
    age: 10
}
