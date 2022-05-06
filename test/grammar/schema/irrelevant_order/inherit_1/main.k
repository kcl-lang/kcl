schema BasePerson:
    fullName = firstName + ' ' + lastName

schema Person(BasePerson):
    firstName = "John"
    lastName: str

JohnDoe = Person {
    lastName: "Doe"
}
