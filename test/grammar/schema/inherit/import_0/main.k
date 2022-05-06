import pkg

schema Scholar(pkg.Person):
    subject: str

JonSnow = pkg.Person {
    "firstName": "Jon",
    "lastName": "Snow"
}

JohnDoe = Scholar {
    "firstName": "John",
    "lastName": "Doe",
    "subject": "CS"
}
