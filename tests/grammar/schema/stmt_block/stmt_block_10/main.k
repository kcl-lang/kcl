schema PersonMixin:
    fullName = "{} {}".format(firstName, lastName)

schema SonMixin:
    sonName = "{} {}".format(firstName, lastName)

schema Person:
    mixin [PersonMixin]

    firstName: str = "John"
    lastName: str = "Doe"

schema Son(Person):
    mixin [SonMixin]

    sonName: str
    firstName: str

AceDoe = Son {}
