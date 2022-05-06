schema Name:
    firstName: str
    lastName: str

schema Person(Name):
    age: int

schema Scholar(Person):
    subject: str
    school: str

JohnReese = Name {
    "firstName": "John",
    "lastName": "Reese"
}

JonSnow = Person {
    "firstName": "Jon",
    "lastName": "Snow",
    "age": 10
}

JohnDoe = Scholar {
    "firstName": "John",
    "lastName": "Doe",
    "age": 10,
    "subject": "CS",
    "school": "PKU"
}
