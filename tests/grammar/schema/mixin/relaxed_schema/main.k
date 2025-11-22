schema Person:
    firstName?: str
    lastName?: str

schema FullnameMixin:
    fullName: str = "{} {}".format(firstName, lastName)

schema Scholar(Person):
    mixin [FullnameMixin]
    school?: str = None

JohnDoe = Scholar {
    "firstName": "John",
    "lastName": "Doe"
}
