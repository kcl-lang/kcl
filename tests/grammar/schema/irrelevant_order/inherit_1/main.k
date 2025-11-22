schema BasePerson:
    firstName: str
    lastName: str
    fullName = firstName + ' ' + lastName

schema Person(BasePerson):
    firstName = "John"
    lastName: str

JohnDoe = Person {
    lastName: "Doe"
}
