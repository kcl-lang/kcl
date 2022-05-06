
schema Person:
    firstName: str = "John"
    lastName: str
    check:
        len(lastName) < 10

schema Student(Person):
    age: int
    check:
        12 <= age <= 18

JohnDoe = Student {
    "lastName": "Doeblablablabla",
    "age": 17
}
