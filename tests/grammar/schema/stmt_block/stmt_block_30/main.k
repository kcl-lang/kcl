schema Parent:
    base: str

person = Person {
    name: "name"
    base: "base"
}

schema Person(Parent):
    name: str
