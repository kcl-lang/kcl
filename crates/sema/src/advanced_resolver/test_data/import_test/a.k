_a = 1
schema Name:
    firstName: str
    lastName: str

schema Person:
    name: Name
    age: int

_person = Person {
    name = Name {
        firstName = "a"
        lastName = "b"
    }
    age = 20
}