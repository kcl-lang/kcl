schema Person:
    firstName: str
    lastName: str
    fullName: str
    _tmp: str = "default"

schema FullnameMixin:
    fullName = "{} {}".format(firstName, lastName)
    _tmp = "fullname_changed"

schema Scholar(Person):
    mixin [FullnameMixin]
    school?: str = None

JohnDoe = Scholar {
    "firstName": "John",
    "lastName": "Doe"
}
