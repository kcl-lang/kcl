import pkg

schema Person:
	name: pkg.Name
	age: int

alice = Person {
    "name": pkg.Name {
        "firstName": "alice",
        "lastName": "smith"
    },
    "age": 10
}