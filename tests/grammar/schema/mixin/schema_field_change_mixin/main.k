schema Person:
    firstName: str
    lastName: str
    fullName: str

schema FullnameMixin:
    fullName = "{} {}".format(firstName, lastName)

schema Scholar(Person):
    mixin [FullnameMixin]
    school?: str = None

JohnDoe = Scholar {
    "firstName" = "John",
    "lastName" = "Doe",
    "fullName" = "Doe Jon"
}
