x0 = Person {
    name.firstName = "Bob"
    name.lastName = "John"
}

schema Person:
    name: Name
    age: int = 1

x1 = Person {
    name.firstName = "Alice"
    name.lastName = "John"
    age = 10
}

schema Name:
    firstName: str
    lastName: str
