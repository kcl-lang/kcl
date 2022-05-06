schema Person:
    name: str = "kcl"
    age: int = 1

schema Data:
    x: Person = {
        age = "123"
    }
