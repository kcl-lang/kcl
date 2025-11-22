schema Person:
    name: str = "Alice"
    nameLower: str = str(name).lower().upper()

person = Person {}
