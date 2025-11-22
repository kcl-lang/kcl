schema FullNameMixin:
    fullName: str
    if _withFull:
        fullName = firstName + ' ' + lastName

schema Person[withFull]:
    mixin [FullNameMixin]

    _withFull: bool = withFull
    firstName: str = "John"
    lastName: str

    mapping: {str:str} = {
        "firstName": firstName
        "lastName": lastName
        "fullName": fullName
    }

JohnDoe = Person(True) {
    lastName: "Doe"
}
