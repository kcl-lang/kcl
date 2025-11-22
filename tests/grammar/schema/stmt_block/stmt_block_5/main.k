schema Person:
    firstName: str = "John"
    lastName: str
    age: int = 12

    if age >= 18:
        lastName = "Doe"

JohnDoe = Person {
    "age" = 18
}
