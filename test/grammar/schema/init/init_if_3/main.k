schema Person:
    name: str
    age: int
    status: str

    if age < 18:
        status = "minor"
    elif age > 70:
        status = "aged"
    else:
        status = "adult"

JohnDoe = Person {
    "name": "John",
    "age": 20
}
