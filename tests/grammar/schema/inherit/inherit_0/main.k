schema Person:
    firstName: str
    lastName: str

schema Scholar(Person):
    subject: str

JonSnow = Person {
    "firstName": "Jon",
    "lastName": "Snow"
}

JohnDoe = Scholar {
    "firstName": "John",
    "lastName": "Doe",
    "subject": "CS"
}
