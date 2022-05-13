schema Name:
    firstName: str
    lastName: str

schema Person:
    name: Name
    age: int

alice = Person {
    name: {
        firstName: "Alice",
        lastName: "Terry"
    },
    age: 12
}
