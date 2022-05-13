schema Data:
    id?: int

schema Person:
    data: Data
    name: a = 1

a: int = 1

person = Person {}
