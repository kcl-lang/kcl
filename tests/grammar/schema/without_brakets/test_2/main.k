schema Person[data, n]:
    name: str = data
    count: int = n

count = Person("Alice", 10).count
