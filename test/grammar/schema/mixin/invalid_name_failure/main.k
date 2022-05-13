schema Person:
    firstName: str
    lastName: str
    fullName: str

schema Fullname:
    fullName = "{} {}".format(firstName, lastName)

schema Scholar(Person):
    mixin [Fullname]
    school: str

JohnDoe = Scholar {
    "firstName": "John",
    "lastName": "Doe",
    "fullName": "Doe Jon"
}
