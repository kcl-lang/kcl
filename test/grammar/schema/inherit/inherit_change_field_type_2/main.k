import pkg

schema Person:
    name: pkg.Name

schema Scholar(Person):
    name: pkg.Name0
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
