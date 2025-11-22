schema Name:
    firstName?: str
    check:
        firstName, "firstName should be defined and not empty"

schema Person:
    name: Name

JohnDoe = Person {
    "name": Name {}
}
