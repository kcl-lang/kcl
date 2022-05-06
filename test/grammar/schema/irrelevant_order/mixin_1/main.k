schema FullNameMixin:
    fullName: str = firstName + ' ' + lastName

schema Person:
    mixin [FullNameMixin]

    firstName: str = "John"
    lastName: str

    firstName = firstName.upper()
    

JohnDoe = Person {
    lastName: "Doe"
}
