schema BasePerson:
    firstName: str = "Alice"
    lastName: str = None
    fullName: str = firstName + ' ' + lastName

schema Person(BasePerson):
    firstName: str = "John"
    lastName: str = "Doe"

schema SubPerson(Person):
    firstName = "Bob"

person = SubPerson {
    firstName = "firstName_override"
    lastName = "lastName_override"
}
