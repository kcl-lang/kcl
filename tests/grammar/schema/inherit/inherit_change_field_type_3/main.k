import pkg

schema Scholar(pkg.Person):
    name: pkg.Name0
    school: str

JohnDoe = Scholar {
    "name": {
        "firstName": "John",
        "lastName": "Doe"
    },
    "school": "PKU"
}
