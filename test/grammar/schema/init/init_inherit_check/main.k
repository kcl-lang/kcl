schema Name:
    firstName: str
    lastName: str
    fullName: str = "{} {}".format(firstName, lastName)

schema Person(Name):
    gender: str

    check:
        fullName not None, "totalName should not be None"

alice = Person {
    "firstName": "Alice",
    "lastName": "Smith",
    "gender": "female"
}
