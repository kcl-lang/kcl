alice_first_name = "alice"

schema Name:
    first: str
    last: str

schema Person:
    name: Name
    name.first = "a"
    name.last = "b"

alice = Person {
    name: {
        first = alice_first_name
        last = "Green"
    }
}
