schema Person:
    firstName: str
    lastName: str

schema KnowledgeMixin:
    firstName: int
    subject: str

schema Scholar(KnowledgeMixin, Person):
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
