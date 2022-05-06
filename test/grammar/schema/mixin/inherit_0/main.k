schema Person:
    firstName: str
    lastName: str
    nameUpper: str

schema UpperMixin:
    nameUpper = firstName.upper()
    schoolUpper = school.upper()

schema Scholar(Person):
    mixin [UpperMixin]
    school: str
    schoolUpper: str

JohnDoe = Scholar {
    "firstName": "John",
    "lastName": "Doe",
    "school": "cs"
}
