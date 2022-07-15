schema Person[data: str, n]:
    name: str = data
    count: int = n

person = Person("Alice", 1) {}
personOther = person as Person
