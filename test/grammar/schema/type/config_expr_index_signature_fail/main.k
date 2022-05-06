schema Name:
    age: int = 10
    [...str]: int

schema Person:
    name: Name

John = Person {
    name: {
        "classID" = "aa"
    }
}
