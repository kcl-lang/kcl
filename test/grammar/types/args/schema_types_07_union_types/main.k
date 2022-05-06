schema Person[data: str, n]:
    name: str = data
    count: int | float | str = n

person = Person(data="Alice", n=1)
