schema Person:
    name: str
    age: int
    status: str = "adult"

    if age < 18:
        status = "minor"
    elif age > 70:
        status = "aged"

JohnDoe = Person {
    "name": "John",
    "age": 10
}
