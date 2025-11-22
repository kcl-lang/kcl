schema Name:
    first?: str
    last?: str

schema Person:
    name?: Name
    age?: int

alice = Person {
    "name": {},
    "age": 10
}
