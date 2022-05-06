
schema Person:
    firstName: str = "John"
    lastName: str
    check:
        len(lastName) > 0

schema Student(Person):
    age: int
    check:
        12 <= age <= 18

JohnDoe = Student {
    "lastName": "Doe",
    "age": 15
}
