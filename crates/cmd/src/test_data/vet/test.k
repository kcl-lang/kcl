schema Person:
    name: str
    age: int
    message: str

    check:
        age > 0
