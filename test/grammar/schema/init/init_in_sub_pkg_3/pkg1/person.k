import ..pkg2

schema Person:
    firstName: str
    lastName: str
    name: pkg2.Name = pkg2.Name {
        "fullName": "{} {}".format(firstName, lastName)
    }
