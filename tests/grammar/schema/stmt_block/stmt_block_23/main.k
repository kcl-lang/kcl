schema PersonMixin:
    headcount: [int] = [i for i in [1, 2, 3]]
    _dict: {str:} = {"key1": "value1", "key2": "value2"}
    keyValue: {str:} = {key: _dict[key] for key in _dict}
    fullName: str = "{} {}".format(firstName, lastName)

schema SonMixin:
    sonName = "{} {}".format(firstName, lastName)

schema Person:
    mixin [PersonMixin]

    firstName: str = "John"
    lastName: str = "Doe"

schema Son(Person):
    mixin [SonMixin]

    sonName: str

AceDoe = Son {}
