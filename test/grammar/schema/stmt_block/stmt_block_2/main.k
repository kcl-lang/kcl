schema Person:
    firstName: str = "John"
    lastName: str
    names: [str] = [firstName, lastName]
    namePair: {str:} = {"${firstName}": lastName}

JohnDoe = Person {
    "lastName": "Doe"
}
