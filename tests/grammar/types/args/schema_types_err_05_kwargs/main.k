schema Person[data: str, n: int]:
    name: str = data
    count: int = n

person = Person(data="Alice", n="1")
