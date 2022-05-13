schema Person:
    name: str
    age: int
    status: str

    if age < 18:
        status = "minor"
    else:
        status = "adult"

JohnDoe = Person {
    "name": "John",
    "age": 10
}
