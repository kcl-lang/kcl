schema Person:
    name: str

    check:
        name, "{}".format(name)

person = Person {
    name: "Alice"
}
