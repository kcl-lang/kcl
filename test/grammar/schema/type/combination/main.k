schema Name:
    firstName: str
    lastName: str

schema Person:
    name: Name
    age: int

alice = Person {
    name: Name {
        firstName: "Alice",
        lastName: "Terry"
    },
    age: 12
}
 