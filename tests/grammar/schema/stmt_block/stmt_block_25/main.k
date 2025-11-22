schema Name:
    name: str

schema Person:
    names?: [Name]

_names = [Name {
    name: "Alice"
}]

_names += [Name {
    name: "Bob"
}]

person = Person {
    names: _names
}
