schema Person:
    firstName?: str = None
    lastName?: str = "Doe"
    prefix?: str = None
    lastName = "{}.{}".format(prefix, lastName) if prefix else lastName

alice = Person {
    "firstName" = "Alice",
    "prefix" = "Mrs"
}

John = Person {
    "lastName" = "Green"
}
