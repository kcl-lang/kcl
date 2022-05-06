schema User:
    name: str
    age: int
    message?: str

    check:
        age > 10
