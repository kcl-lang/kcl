schema Person:
    firstName: str
    lastName: str

schema Knowledge:
    firstName: int
    subject: str

schema Scholar(Person, Knowledge):
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
