schema Person:
    firstName: str
    lastName: str

schema Scholar(Person):
    firstName: int
    subject: str
    school: str

JonSnow = Person {
    "firstName": "Jon",
    "lastName": "Snow"
}

JohnDoe = Scholar {
    "firstName": "John",
    "lastName": "Doe",
    "subject": "CS",
    "school": "PKU"
}
