
schema PersonMixin:
    fullName: str = "{} {}".format(firstName, lastName)
    # print("PersonMixin")

schema SonMixin:
    sonName: str = "{} {}".format(firstName, lastName)
    # print("SonMixin")

schema Person:
    mixin [PersonMixin]
    # print("PersonStmt")
    firstName: str = "John"
    lastName: str = "Doe"

schema Son(Person):
    mixin [SonMixin]
    # print("SonStmt")
    sonName: str
    firstName: str

AceDoe = Son {}
# print("===")
