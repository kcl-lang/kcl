schema Person:
    firstName: str = "John"
    midName: str = "Spam"
    lastName: str
    age: int
    check:
        age < 140, "age is too large"

JohnDoe = Person {
    "lastName": "Doe",
    "age": 1000
}
