person = Person {
    name: "name"
    base: "base"
}

schema Person(Parent):
    name: str

schema Parent:
    base: str
