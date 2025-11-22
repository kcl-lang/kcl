schema FullNameMixin:
    fullName: str = (firstName + ' ' + lastName) if _withFull else None

schema Person[withFull]:
    mixin [FullNameMixin]

    _withFull: bool = withFull
    firstName: str = "John"
    lastName: str
    fullName: str

    mapping: {str:str} = {
        "firstName": firstName
        "lastName": lastName
        "fullName": fullName
    }

JohnDoe = Person(True) {
    lastName: "Doe"
}
