schema Name:
    name: str

schema Person:
    names: [str] = [name for name in ["Alice", "Bob", "John"]]
    name: Name = Name {
        name: names[0]
    }

person = Person {}
