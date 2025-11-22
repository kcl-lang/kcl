schema Person:
    firstName: str
    lastName: str

schema FullnameMixin:
    fullName = "{} {}".format(firstName, lastName)

schema Scholar(FullnameMixin):
    school: str

JohnDoe = Scholar {
    "firstName": "Jon",
    "lastName": "Doe"
}
