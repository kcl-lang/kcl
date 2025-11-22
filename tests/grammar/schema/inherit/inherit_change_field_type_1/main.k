import pkg

schema Person:
    name: pkg.Name

schema Scholar(Person):
    name: pkg.Name
    school: str

JonSnow = Person {
    "name": {
        "firstName": "Jon",
        "lastName": "Snow"
    }
}

JohnDoe = Scholar {
    "name": {
        "firstName": "John",
        "lastName": "Doe"
    },
    "school": "PKU"
}
