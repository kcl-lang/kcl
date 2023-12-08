schema User:
    name: str
    age: int
    message?: str

    check:
        name == "Alice"
        age > 10
