schema Person(Parent):
    name: str

schema Parent:
    base: str

person = Person {
    name: "name"
    base: "base"
}
