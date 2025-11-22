schema Person:
    name: str
    age: int
    status: str

    if age > 18:
        if age < 70:
            status = "adult"
        else:
            status = "aged"
    else:
        status = "minor"

JohnDoe = Person {
    "name": "John",
    "age": 80
}
