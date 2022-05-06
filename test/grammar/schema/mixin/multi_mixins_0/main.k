schema Person:
    firstName: str
    lastName: str
    fullName: str
    upper: str

schema FullnameMixin:
    fullName = "{} {}".format(firstName, lastName)

schema UpperMixin:
    upper = fullName.upper()

schema Scholar(Person):
    mixin [FullnameMixin, UpperMixin]
    school?: str = None

JohnDoe = Scholar {
    "firstName": "John",
    "lastName": "Doe"
}
