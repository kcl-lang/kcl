schema Name:
    first?: str = None
    last?: str = None

schema Person:
    name?: Name
    age?: int

alice = Person {
    "name": {},
    "age": 10
}
