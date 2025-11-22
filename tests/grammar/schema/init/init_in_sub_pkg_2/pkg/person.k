schema Person:
    firstName: str
    lastName: str
    name: Name = Name {
        "fullName": "{} {}".format(firstName, lastName)
    }
