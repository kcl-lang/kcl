import pkg

schema Scholar(pkg.Person):
    name: pkg.Name
    school: str

JohnDoe = Scholar {
    "name": {
        "firstName": "John",
        "lastName": "Doe"
    },
    "school": "PKU"
}
