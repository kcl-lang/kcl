schema Person:
    name: str

    check:
        name.istitle(), "name of person must be titled"

alice = Person {
    "name": "Alice"
}
