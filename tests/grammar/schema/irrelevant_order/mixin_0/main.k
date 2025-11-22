schema FullNameMixin:
    fullName: str = firstName + ' ' + lastName

schema Person:
    mixin [FullNameMixin]

    firstName: str = "John"
    lastName: str
    

JohnDoe = Person {
    lastName: "Doe"
}
