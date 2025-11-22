schema Person[data: str, n]:
    name: str = data
    count: int = n

person = Person(data="Alice", n=1)
