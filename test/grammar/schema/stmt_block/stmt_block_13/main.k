schema Person:
    name: str
    age: int
    status: str

    _status_aged = "aged"
    _status_adult = "adult"
    _status_minor = "minor"

    if age > 18:
        if age < 70:
            status = _status_adult
        else:
            status = _status_aged
    else:
        status = _status_minor

JohnDoe = Person {
    "name": "John",
    "age": 80
}
