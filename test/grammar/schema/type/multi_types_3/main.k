schema Name:
    first: str
    second: str | bool

schema Person(Name):
    age: int

aliceGreen = Person {
    first: "alice",
    second: "Green",
    age: 10
}

alice = Person {
    first: "alice",
    second: False,
    age: 10
}
