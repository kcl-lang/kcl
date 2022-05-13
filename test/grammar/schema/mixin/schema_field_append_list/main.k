schema Person:
    firstName: str
    lastName: str
    fullName: str
    tags: [str]

schema InfoMixin:
    tags += ["234"]

schema Scholar(Person):
    mixin [InfoMixin]
    school?: str = None

JohnDoe = Scholar {
    "firstName": "John",
    "lastName": "Doe",
    "fullName": "Doe Jon",
    "tags" = ["123"]
}
