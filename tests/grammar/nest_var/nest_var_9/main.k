schema Person:
    [str]: str
    name: str = "Alice"

person = {
    "name": "Alice"
}

data = Person {
    person["name"]: person["name"].upper()
}
